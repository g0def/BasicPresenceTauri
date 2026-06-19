use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::profile_bundle_dto::{
    BundleImportSelectionDto, BundleImportSummaryDto, BundleImportTargetDto,
};
use crate::application::dto::profile_settings_dto::ProfileSettingsDto;
use crate::application::use_cases::create_profile::{normalize_optional, validate_required};
use crate::application::use_cases::create_task_preset::{
    normalize_description, validate_color, validate_minutes, validate_title,
};
use crate::application::use_cases::set_presence::validate_day;
use crate::application::use_cases::set_profile_settings::validate as validate_settings;
use crate::application::use_cases::set_work_schedule::validate_start;
use crate::domain::entities::commute::{Commute, CommuteSegment};
use crate::domain::entities::presence::{Presence, PresenceType};
use crate::domain::entities::profile::Profile;
use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::entities::task_preset::TaskPreset;
use crate::domain::entities::trip::Trip;
use crate::domain::entities::work_day_schedule::WorkDaySchedule;
use crate::domain::entities::work_entry::WorkEntry;
use crate::domain::error::DomainError;
use crate::domain::repositories::commute_repository::CommuteRepository;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::domain::repositories::profile_settings_repository::ProfileSettingsRepository;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::services::clock::Clock;
use crate::domain::services::profile_bundle::{
    BundleDay, ParsedBundle, ProfileBundleCodec, SUPPORTED_BUNDLE_VERSION,
};

/// A day validated and ready to write. Work-entry/schedule `presence_id`s are
/// filled in only after the presence upsert returns the persisted id (which, on
/// a merge-replace, is the *existing* row's id, not a freshly minted one).
struct PreparedDay {
    day: i64,
    kind: PresenceType,
    co2_kg: Option<f64>,
    is_estimated: bool,
    created_at: i64,
    updated_at: i64,
    trips: Vec<Trip>,
    work_entries: Vec<WorkEntry>,
    schedule: Option<WorkDaySchedule>,
    note: Option<String>,
}

/// Import a shared profile bundle, either into a fresh profile or merged into an
/// existing one. The file is read+parsed by the [`ProfileBundleCodec`]; the whole
/// parsed content is validated up front (so a malformed file aborts before any
/// write); the writes then reuse the existing per-entity repository methods,
/// preserving frozen CO₂ snapshots and timestamps verbatim. On a merge that
/// *replaces* an existing day, the upsert keeps that row's original `created_at`
/// (only fresh inserts carry the bundle's `created_at` through).
pub struct ImportProfileBundleUseCase {
    codec: Arc<dyn ProfileBundleCodec>,
    profiles: Arc<dyn ProfileRepository>,
    presences: Arc<dyn PresenceRepository>,
    work_entries: Arc<dyn WorkEntryRepository>,
    task_presets: Arc<dyn TaskPresetRepository>,
    commutes: Arc<dyn CommuteRepository>,
    profile_settings: Arc<dyn ProfileSettingsRepository>,
    clock: Arc<dyn Clock>,
}

impl ImportProfileBundleUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        codec: Arc<dyn ProfileBundleCodec>,
        profiles: Arc<dyn ProfileRepository>,
        presences: Arc<dyn PresenceRepository>,
        work_entries: Arc<dyn WorkEntryRepository>,
        task_presets: Arc<dyn TaskPresetRepository>,
        commutes: Arc<dyn CommuteRepository>,
        profile_settings: Arc<dyn ProfileSettingsRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            codec,
            profiles,
            presences,
            work_entries,
            task_presets,
            commutes,
            profile_settings,
            clock,
        }
    }

    pub async fn execute(
        &self,
        path: &str,
        selection: BundleImportSelectionDto,
        target: BundleImportTargetDto,
        conflict_strategy: &str,
    ) -> Result<BundleImportSummaryDto, DomainError> {
        let path = path.trim();
        if path.is_empty() {
            return Err(DomainError::Validation("path is required".to_string()));
        }
        let bundle = self.codec.read(Path::new(path))?;
        if bundle.version > SUPPORTED_BUNDLE_VERSION {
            return Err(DomainError::Validation(format!(
                "this file was created by a newer version (v{}) and cannot be imported",
                bundle.version
            )));
        }
        let replace = match conflict_strategy {
            "skip" => false,
            "replace" => true,
            other => {
                return Err(DomainError::Validation(format!(
                    "invalid conflict strategy: {other}"
                )))
            }
        };

        // --- Validation pass: build domain objects without touching the vault. ---
        let prepared_days = if selection.include_days {
            prepare_days(&bundle.days, selection)?
        } else {
            Vec::new()
        };
        let prepared_presets = if selection.include_task_presets {
            prepare_presets(&bundle.task_presets)?
        } else {
            Vec::new()
        };
        let prepared_commutes = if selection.include_commutes {
            prepare_commutes(&bundle.commutes)?
        } else {
            Vec::new()
        };
        let prepared_settings: Option<ProfileSettings> = if selection.include_settings {
            if let Some(s) = &bundle.settings {
                // Untrusted: enforce the same field rules (enums/ranges) as
                // SetProfileSettingsUseCase so a corrupt settings block aborts here,
                // up front, with a clean Validation error — instead of an opaque
                // Storage error mid-import after other categories were written.
                validate_settings(&ProfileSettingsDto::from(s.clone()))?;
            }
            bundle.settings.clone()
        } else {
            None
        };

        // --- Resolve the target profile. ---
        let now = self.clock.now_ms();
        let mut existing_days: HashSet<i64> = HashSet::new();
        let (target_profile_id, is_merge) = match target.kind.as_str() {
            "new" => {
                let target_id = self.create_target_profile(&bundle, &target, now).await?;
                (target_id, false)
            }
            "existing" => {
                let pid = target
                    .profile_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        DomainError::Validation(
                            "profileId is required for an existing target".to_string(),
                        )
                    })?;
                self.profiles
                    .find_by_id(pid)
                    .await?
                    .ok_or(DomainError::ProfileNotFound)?;
                for p in self.presences.list_by_profile(pid).await? {
                    existing_days.insert(p.day);
                }
                (pid.to_string(), true)
            }
            other => {
                return Err(DomainError::Validation(format!(
                    "invalid import target: {other}"
                )))
            }
        };

        // --- Write pass. ---
        let mut summary = BundleImportSummaryDto {
            profile_id: target_profile_id.clone(),
            days_imported: 0,
            days_replaced: 0,
            days_skipped: 0,
            trips: 0,
            work_entries: 0,
            notes: 0,
            task_presets: 0,
            commutes: 0,
            settings: false,
        };

        for mut pd in prepared_days {
            let conflict = is_merge && existing_days.contains(&pd.day);
            if conflict && !replace {
                summary.days_skipped += 1;
                continue;
            }

            let presence = Presence {
                id: Uuid::now_v7().to_string(),
                profile_id: target_profile_id.clone(),
                day: pd.day,
                kind: pd.kind,
                co2_kg: pd.co2_kg,
                is_estimated: pd.is_estimated,
                created_at: pd.created_at,
                updated_at: pd.updated_at,
                work_minutes: 0,
            };
            let saved = self.presences.set_for_day(&presence, &pd.trips).await?;
            summary.trips += pd.trips.len() as u32;
            if conflict {
                summary.days_replaced += 1;
            } else {
                summary.days_imported += 1;
            }

            if selection.include_work_hours {
                // Replace-all: writes the imported entries (empty for a non-work day)
                // and clears any stale entries when replacing a day.
                for e in &mut pd.work_entries {
                    e.presence_id = saved.id.clone();
                }
                self.work_entries
                    .replace_for_presence(&saved.id, &pd.work_entries)
                    .await?;
                summary.work_entries += pd.work_entries.len() as u32;
                match pd.schedule.take() {
                    Some(mut schedule) => {
                        schedule.presence_id = saved.id.clone();
                        // end is DERIVED (start + sum of entry minutes), never the
                        // file's value, so the stored schedule stays self-consistent.
                        let total: i64 = pd.work_entries.iter().map(|e| e.minutes).sum();
                        schedule.end_minutes = schedule.start_minutes + total;
                        self.work_entries.set_schedule(&schedule).await?;
                    }
                    None => {
                        // No schedule for this day: drop any the replaced day carried,
                        // so a "replace" can't leave the previous day's start/end.
                        self.work_entries.clear_schedule(&saved.id).await?;
                    }
                }
            }

            if selection.include_notes {
                // Faithful: sets the note, or clears it (None) on a replaced day.
                self.presences
                    .set_note(&saved.id, pd.note.as_deref(), pd.updated_at)
                    .await?;
                if pd.note.is_some() {
                    summary.notes += 1;
                }
            }
        }

        // On a merge, skip presets/commutes whose name already exists on the target,
        // so re-importing the same bundle doesn't accumulate duplicates. Unlike days
        // (UNIQUE per profil), presets/commutes have no natural key, so they're
        // deduped here by their user-facing name. The summary counts what was
        // actually created. (Nothing pre-exists for a "new" profile.)
        let existing_preset_titles: HashSet<String> = if is_merge {
            self.task_presets
                .list_by_profile(&target_profile_id)
                .await?
                .into_iter()
                .map(|p| p.title)
                .collect()
        } else {
            HashSet::new()
        };
        for mut preset in prepared_presets {
            if existing_preset_titles.contains(&preset.title) {
                continue;
            }
            preset.profile_id = target_profile_id.clone();
            self.task_presets.create(&preset).await?;
            summary.task_presets += 1;
        }

        let existing_commute_names: HashSet<String> = if is_merge {
            self.commutes
                .list_by_profile(&target_profile_id)
                .await?
                .into_iter()
                .map(|c| c.name)
                .collect()
        } else {
            HashSet::new()
        };
        for mut commute in prepared_commutes {
            if existing_commute_names.contains(&commute.name) {
                continue;
            }
            commute.profile_id = target_profile_id.clone();
            self.commutes.create(&commute).await?;
            summary.commutes += 1;
        }

        if let Some(settings) = prepared_settings {
            self.profile_settings
                .save(&target_profile_id, &settings)
                .await?;
            summary.settings = true;
        }

        Ok(summary)
    }

    /// Create the destination profile for a "new" import (identity from the
    /// override fields, falling back to the bundle's), set it active, return its id.
    async fn create_target_profile(
        &self,
        bundle: &ParsedBundle,
        target: &BundleImportTargetDto,
        now: i64,
    ) -> Result<String, DomainError> {
        let first_name = target
            .first_name
            .as_deref()
            .unwrap_or(&bundle.profile.first_name)
            .trim();
        let last_name = target
            .last_name
            .as_deref()
            .unwrap_or(&bundle.profile.last_name)
            .trim();
        let enterprise = target
            .enterprise
            .as_deref()
            .unwrap_or(&bundle.profile.enterprise)
            .trim();
        let poste = target.poste.as_deref().or(bundle.profile.poste.as_deref());

        validate_required("firstName", first_name)?;
        validate_required("lastName", last_name)?;
        validate_required("enterprise", enterprise)?;
        let poste = normalize_optional("poste", poste)?;

        let profile = Profile {
            id: Uuid::now_v7().to_string(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            enterprise: enterprise.to_string(),
            poste,
            created_at: now,
            updated_at: now,
        };
        self.profiles.create(&profile).await?;
        // Surface the freshly imported profile (it becomes the active one).
        self.profiles.set_active(Some(&profile.id)).await?;
        Ok(profile.id)
    }
}

fn prepare_days(
    days: &[BundleDay],
    selection: BundleImportSelectionDto,
) -> Result<Vec<PreparedDay>, DomainError> {
    let mut out = Vec::with_capacity(days.len());
    for d in days {
        // Untrusted file content: the vault has no CHECK on `day`, so enforce the
        // same rule as the normal write path (epoch ms at UTC midnight).
        validate_day(d.presence.day)?;

        // CO2/trips/work hours are tied to presence: only office/remote days carry
        // them, and other types keep a NULL footprint (set_presence enforces this).
        // Mirror it here so a hand-edited bundle can't persist a state the rest of
        // the app deems impossible (a footprint or work hours on a holiday).
        let is_work = matches!(d.presence.kind, PresenceType::Office | PresenceType::Remote);

        let co2_kg = if is_work {
            validate_co2(d.presence.co2_kg)?
        } else {
            None
        };

        let trips: Vec<Trip> = if selection.include_trips && is_work {
            let mut trips = Vec::with_capacity(d.trips.len());
            for t in &d.trips {
                if t.distance_km < 0.0 || !t.distance_km.is_finite() {
                    return Err(DomainError::Validation(
                        "trip distance must be a finite value >= 0".to_string(),
                    ));
                }
                validate_co2(Some(t.co2_kg))?;
                trips.push(Trip {
                    id: Uuid::now_v7().to_string(),
                    mode_id: t.mode_id.clone(),
                    distance_km: t.distance_km,
                    round_trip: t.round_trip,
                    occupants: t.occupants.max(1),
                    co2_kg: t.co2_kg,
                    is_estimated: t.is_estimated,
                    factor_year: t.factor_year,
                    position: t.position,
                });
            }
            trips
        } else {
            Vec::new()
        };

        let mut work_entries = Vec::new();
        let mut schedule = None;
        if selection.include_work_hours && is_work {
            for e in &d.work_entries {
                work_entries.push(WorkEntry {
                    id: Uuid::now_v7().to_string(),
                    presence_id: String::new(),
                    title: validate_title(&e.title)?,
                    description: normalize_description(e.description.clone()),
                    minutes: validate_minutes(e.minutes)?,
                    color: validate_color(&e.color)?,
                    position: e.position,
                });
            }
            schedule = match d.schedule.as_ref() {
                Some(s) => {
                    // start is bounded like the normal path; end is DERIVED at write
                    // time (start + sum of entry minutes), never trusted from the file.
                    validate_start(s.start_minutes)?;
                    Some(WorkDaySchedule {
                        presence_id: String::new(),
                        start_minutes: s.start_minutes,
                        end_minutes: 0,
                    })
                }
                None => None,
            };
        }

        let note = if selection.include_notes {
            d.note
                .as_ref()
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty())
        } else {
            None
        };

        out.push(PreparedDay {
            day: d.presence.day,
            kind: d.presence.kind,
            co2_kg,
            is_estimated: d.presence.is_estimated,
            created_at: d.presence.created_at,
            updated_at: d.presence.updated_at,
            trips,
            work_entries,
            schedule,
            note,
        });
    }
    Ok(out)
}

/// A presence/trip `co2_kg`, when present, must be finite and non-negative — a
/// hand-edited bundle could carry NaN/Inf/negative and SQLite would store it,
/// poisoning later CO2 aggregation.
fn validate_co2(co2_kg: Option<f64>) -> Result<Option<f64>, DomainError> {
    if let Some(c) = co2_kg {
        if !c.is_finite() || c < 0.0 {
            return Err(DomainError::Validation(
                "co2 must be a finite value >= 0".to_string(),
            ));
        }
    }
    Ok(co2_kg)
}

fn prepare_presets(presets: &[TaskPreset]) -> Result<Vec<TaskPreset>, DomainError> {
    let mut out = Vec::with_capacity(presets.len());
    for p in presets {
        out.push(TaskPreset {
            id: Uuid::now_v7().to_string(),
            profile_id: String::new(),
            title: validate_title(&p.title)?,
            description: normalize_description(p.description.clone()),
            default_minutes: validate_minutes(p.default_minutes)?,
            color: validate_color(&p.color)?,
            created_at: p.created_at,
            updated_at: p.updated_at,
        });
    }
    Ok(out)
}

fn prepare_commutes(commutes: &[Commute]) -> Result<Vec<Commute>, DomainError> {
    let mut out = Vec::with_capacity(commutes.len());
    for c in commutes {
        let name = c.name.trim();
        if name.is_empty() {
            return Err(DomainError::Validation(
                "commute name is required".to_string(),
            ));
        }
        let commute_id = Uuid::now_v7().to_string();
        let mut segments = Vec::with_capacity(c.segments.len());
        for (i, s) in c.segments.iter().enumerate() {
            let mode_id = s.mode_id.trim();
            if mode_id.is_empty() {
                return Err(DomainError::Validation(format!(
                    "commute segment {i}: mode is required"
                )));
            }
            if s.distance_km < 0.0 || !s.distance_km.is_finite() {
                return Err(DomainError::Validation(format!(
                    "commute segment {i}: distance must be >= 0"
                )));
            }
            segments.push(CommuteSegment {
                id: Uuid::now_v7().to_string(),
                commute_id: commute_id.clone(),
                mode_id: mode_id.to_string(),
                distance_km: s.distance_km,
                occupants: s.occupants.max(1),
                position: s.position,
            });
        }
        if segments.is_empty() {
            return Err(DomainError::Validation(
                "a commute must have at least one segment".to_string(),
            ));
        }
        out.push(Commute {
            id: commute_id,
            profile_id: String::new(),
            name: name.to_string(),
            round_trip: c.round_trip,
            segments,
            created_at: c.created_at,
            updated_at: c.updated_at,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_co2_accepts_none_and_finite_nonneg() {
        assert!(validate_co2(None).is_ok());
        assert!(validate_co2(Some(0.0)).is_ok());
        assert!(validate_co2(Some(12.5)).is_ok());
    }

    #[test]
    fn validate_co2_rejects_negative_and_non_finite() {
        assert!(validate_co2(Some(-0.1)).is_err());
        assert!(validate_co2(Some(f64::NAN)).is_err());
        assert!(validate_co2(Some(f64::INFINITY)).is_err());
    }
}
