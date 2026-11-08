//! Validation pass of the bundle import: turns untrusted file content into
//! domain objects ready to write, without touching the vault. Every rule is the
//! one the regular commands enforce (`domain::rules`), so a hand-edited bundle
//! cannot persist a state the rest of the app deems impossible.

use uuid::Uuid;

use crate::application::dto::profile_bundle_dto::BundleImportSelectionDto;
use crate::application::services::commute_builder::{SegmentInput, build_commute};
use crate::domain::entities::commute::Commute;
use crate::domain::entities::presence::PresenceType;
use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::entities::task_preset::TaskPreset;
use crate::domain::entities::trip::Trip;
use crate::domain::entities::work_day_schedule::WorkDaySchedule;
use crate::domain::entities::work_entry::WorkEntry;
use crate::domain::error::DomainError;
use crate::domain::rules;
use crate::domain::services::profile_bundle::BundleDay;

/// A day validated and ready to write. Work-entry/schedule `presence_id`s are
/// filled in only after the presence upsert returns the persisted id (which, on
/// a merge-replace, is the *existing* row's id, not a freshly minted one).
pub(super) struct PreparedDay {
    pub day: i64,
    pub kind: PresenceType,
    pub co2_kg: Option<f64>,
    pub is_estimated: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub trips: Vec<Trip>,
    pub work_entries: Vec<WorkEntry>,
    pub schedule: Option<WorkDaySchedule>,
    pub note: Option<String>,
}

/// Owner id given to prepared commutes until the target profile is known.
const PENDING_OWNER: &str = "pending-import-target";

pub(super) fn days(
    days: &[BundleDay],
    selection: BundleImportSelectionDto,
) -> Result<Vec<PreparedDay>, DomainError> {
    days.iter().map(|d| day(d, selection)).collect()
}

fn day(d: &BundleDay, selection: BundleImportSelectionDto) -> Result<PreparedDay, DomainError> {
    let day = rules::day(d.presence.day)?;

    // CO2/trips/work hours are tied to presence: only office/remote days carry
    // them, and other types keep a NULL footprint (set_presence enforces this).
    let is_work = matches!(d.presence.kind, PresenceType::Office | PresenceType::Remote);
    let co2_kg = if is_work {
        rules::co2_kg(d.presence.co2_kg)?
    } else {
        None
    };

    let trips = if selection.include_trips && is_work {
        rules::segment_count(d.trips.len(), false)?;
        d.trips
            .iter()
            .enumerate()
            .map(|(i, t)| {
                Ok(Trip {
                    id: Uuid::now_v7().to_string(),
                    mode_id: rules::mode_id(i, &t.mode_id)?,
                    distance_km: rules::distance_km(i, t.distance_km)?,
                    round_trip: t.round_trip,
                    occupants: rules::occupants(i, Some(t.occupants))?,
                    co2_kg: rules::co2_kg(Some(t.co2_kg))?.unwrap_or_default(),
                    is_estimated: t.is_estimated,
                    factor_year: rules::factor_year(i64::from(t.factor_year))?,
                    position: t.position,
                })
            })
            .collect::<Result<Vec<_>, DomainError>>()?
    } else {
        Vec::new()
    };

    let mut work_entries = Vec::new();
    let mut schedule = None;
    if selection.include_work_hours && is_work {
        rules::work_day_total(&d.work_entries.iter().map(|e| e.minutes).collect::<Vec<_>>())?;
        for e in &d.work_entries {
            work_entries.push(WorkEntry {
                id: Uuid::now_v7().to_string(),
                presence_id: String::new(),
                title: rules::title(&e.title)?,
                description: rules::description(e.description.as_deref())?,
                minutes: rules::task_minutes(e.minutes)?,
                color: rules::color(&e.color)?,
                position: e.position,
            });
        }
        // The start is bounded like the normal path; the end is DERIVED at
        // write time (start + sum of entry minutes), never trusted from the file.
        schedule = d
            .schedule
            .as_ref()
            .map(|s| {
                Ok::<_, DomainError>(WorkDaySchedule {
                    presence_id: String::new(),
                    start_minutes: rules::start_minutes(s.start_minutes)?,
                    end_minutes: 0,
                })
            })
            .transpose()?;
    }

    let note = if selection.include_notes {
        rules::note(d.note.as_deref())?
    } else {
        None
    };

    Ok(PreparedDay {
        day,
        kind: d.presence.kind,
        co2_kg,
        is_estimated: d.presence.is_estimated,
        created_at: d.presence.created_at,
        updated_at: d.presence.updated_at,
        trips,
        work_entries,
        schedule,
        note,
    })
}

pub(super) fn presets(presets: &[TaskPreset]) -> Result<Vec<TaskPreset>, DomainError> {
    presets
        .iter()
        .map(|p| {
            Ok(TaskPreset {
                id: Uuid::now_v7().to_string(),
                profile_id: String::new(),
                title: rules::title(&p.title)?,
                description: rules::description(p.description.as_deref())?,
                default_minutes: rules::task_minutes(p.default_minutes)?,
                color: rules::color(&p.color)?,
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
        })
        .collect()
}

/// Commutes go through the same builder as create/update; the owner is set
/// once the target profile is resolved.
pub(super) fn commutes(commutes: &[Commute]) -> Result<Vec<Commute>, DomainError> {
    commutes
        .iter()
        .map(|c| {
            let mut segments: Vec<_> = c.segments.iter().collect();
            segments.sort_by_key(|s| s.position);
            build_commute(
                Uuid::now_v7().to_string(),
                PENDING_OWNER,
                &c.name,
                c.round_trip,
                segments.into_iter().map(|s| SegmentInput {
                    mode_id: &s.mode_id,
                    distance_km: s.distance_km,
                    occupants: Some(s.occupants),
                }),
                c.created_at,
                c.updated_at,
            )
        })
        .collect()
}

pub(super) fn settings(settings: &ProfileSettings) -> Result<ProfileSettings, DomainError> {
    rules::profile_settings(settings)?;
    Ok(settings.clone())
}
