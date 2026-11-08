mod prepare;

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::profile_bundle_dto::{
    BundleImportSelectionDto, BundleImportSummaryDto, BundleImportTargetDto, ConflictStrategy,
    ImportTargetKind,
};
use crate::application::services::count;
use crate::domain::entities::presence::Presence;
use crate::domain::entities::profile::Profile;
use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::error::DomainError;
use crate::domain::repositories::commute_repository::CommuteRepository;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::domain::repositories::profile_settings_repository::ProfileSettingsRepository;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::rules;
use crate::domain::services::clock::Clock;
use crate::domain::services::profile_bundle::{
    ParsedBundle, ProfileBundleCodec, SUPPORTED_BUNDLE_VERSION,
};
use crate::domain::services::unit_of_work::UnitOfWork;

/// Import a shared profile bundle, either into a fresh profile or merged into an
/// existing one. The file is read+parsed by the [`ProfileBundleCodec`]; the whole
/// parsed content is validated up front (so a malformed file aborts before any
/// write); the writes then reuse the existing per-entity repository methods,
/// preserving frozen CO₂ snapshots and timestamps verbatim, and run as ONE
/// transaction: an import either lands completely or not at all. On a merge that
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
    uow: Arc<dyn UnitOfWork>,
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
        uow: Arc<dyn UnitOfWork>,
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
            uow,
        }
    }

    pub async fn execute(
        &self,
        path: &Path,
        selection: BundleImportSelectionDto,
        target: BundleImportTargetDto,
        conflict_strategy: ConflictStrategy,
    ) -> Result<BundleImportSummaryDto, DomainError> {
        let bundle = self.codec.read(path)?;
        if bundle.version > SUPPORTED_BUNDLE_VERSION {
            return Err(DomainError::Validation(format!(
                "this file was created by a newer version (v{}) and cannot be imported",
                bundle.version
            )));
        }
        let replace = conflict_strategy == ConflictStrategy::Replace;

        // --- Validation pass: build domain objects without touching the vault. ---
        let prepared_days = if selection.include_days {
            prepare::days(&bundle.days, selection)?
        } else {
            Vec::new()
        };
        let prepared_presets = if selection.include_task_presets {
            prepare::presets(&bundle.task_presets)?
        } else {
            Vec::new()
        };
        let prepared_commutes = if selection.include_commutes {
            prepare::commutes(&bundle.commutes)?
        } else {
            Vec::new()
        };
        let prepared_settings: Option<ProfileSettings> = match &bundle.settings {
            Some(s) if selection.include_settings => Some(prepare::settings(s)?),
            _ => None,
        };

        // --- Write pass, as one transaction. ---
        let mut summary = None;
        self.uow
            .run(Box::pin(async {
                summary = Some(
                    self.write(
                        &bundle,
                        selection,
                        &target,
                        replace,
                        prepared_days,
                        prepared_presets,
                        prepared_commutes,
                        prepared_settings,
                    )
                    .await?,
                );
                Ok(())
            }))
            .await?;
        summary.ok_or_else(|| DomainError::Storage("import produced no summary".into()))
    }

    #[allow(clippy::too_many_arguments)]
    async fn write(
        &self,
        bundle: &ParsedBundle,
        selection: BundleImportSelectionDto,
        target: &BundleImportTargetDto,
        replace: bool,
        prepared_days: Vec<prepare::PreparedDay>,
        prepared_presets: Vec<crate::domain::entities::task_preset::TaskPreset>,
        prepared_commutes: Vec<crate::domain::entities::commute::Commute>,
        prepared_settings: Option<ProfileSettings>,
    ) -> Result<BundleImportSummaryDto, DomainError> {
        // --- Resolve the target profile. ---
        let now = self.clock.now_ms();
        let mut existing_days: HashSet<i64> = HashSet::new();
        let (target_profile_id, is_merge) = match target.kind {
            ImportTargetKind::New => {
                let target_id = self.create_target_profile(bundle, target, now).await?;
                (target_id, false)
            }
            ImportTargetKind::Existing => {
                let pid = rules::required_id(
                    "profileId",
                    target.profile_id.as_deref().unwrap_or_default(),
                )?;
                self.profiles
                    .find_by_id(pid)
                    .await?
                    .ok_or(DomainError::ProfileNotFound)?;
                for p in self.presences.list_by_profile(pid).await? {
                    existing_days.insert(p.day);
                }
                (pid.to_string(), true)
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
            summary.trips += count(pd.trips.len());
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
                summary.work_entries += count(pd.work_entries.len());
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
        let first_name = rules::profile_field(
            "firstName",
            target
                .first_name
                .as_deref()
                .unwrap_or(&bundle.profile.first_name),
        )?;
        let last_name = rules::profile_field(
            "lastName",
            target
                .last_name
                .as_deref()
                .unwrap_or(&bundle.profile.last_name),
        )?;
        let enterprise = rules::profile_field(
            "enterprise",
            target
                .enterprise
                .as_deref()
                .unwrap_or(&bundle.profile.enterprise),
        )?;
        let poste = rules::profile_optional_field(
            "poste",
            target.poste.as_deref().or(bundle.profile.poste.as_deref()),
        )?;

        let profile = Profile {
            id: Uuid::now_v7().to_string(),
            first_name,
            last_name,
            enterprise,
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
