use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::presence_dto::PresenceDto;
use crate::application::dto::trip_dto::TripInputDto;
use crate::application::use_cases::factor_maps::load_factor_maps;
use crate::domain::entities::presence::{Presence, PresenceType};
use crate::domain::entities::trip::{Trip, TripInput};
use crate::domain::entities::work_day_schedule::WorkDaySchedule;
use crate::domain::error::DomainError;
use crate::domain::repositories::emission_factor_repository::EmissionFactorRepository;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::profile_settings_repository::ProfileSettingsRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::services::clock::Clock;
use crate::domain::services::co2_calculator::Co2Calculator;

/// Milliseconds in a UTC day. A valid `day` is a non-negative multiple of it
/// (epoch ms at UTC midnight), matching the key the frontend sends.
const MS_PER_DAY: i64 = 86_400_000;

/// Set (create or update) the presence type for a given day of a profile, and
/// compute + snapshot the day's commute footprint. Re-setting the same day
/// overwrites the type/trips rather than duplicating the row. CO2 is tied to
/// presence: only office/remote days carry trips; other types clear them.
///
/// On first creation of an office/remote day it also seeds the work-day
/// schedule with the profile's default start time, so the work-hours page is
/// pre-filled instead of blank. CO2 is computed under the profile's own config.
pub struct SetPresenceUseCase {
    presences: Arc<dyn PresenceRepository>,
    factors: Arc<dyn EmissionFactorRepository>,
    profile_settings: Arc<dyn ProfileSettingsRepository>,
    entries: Arc<dyn WorkEntryRepository>,
    clock: Arc<dyn Clock>,
}

impl SetPresenceUseCase {
    pub fn new(
        presences: Arc<dyn PresenceRepository>,
        factors: Arc<dyn EmissionFactorRepository>,
        profile_settings: Arc<dyn ProfileSettingsRepository>,
        entries: Arc<dyn WorkEntryRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            presences,
            factors,
            profile_settings,
            entries,
            clock,
        }
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        day: i64,
        kind: &str,
        trips: Vec<TripInputDto>,
    ) -> Result<PresenceDto, DomainError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(DomainError::Validation("profileId is required".to_string()));
        }
        if day < 0 || day % MS_PER_DAY != 0 {
            return Err(DomainError::Validation(
                "day must be epoch ms at UTC midnight".to_string(),
            ));
        }
        let kind = PresenceType::parse(kind)?;

        // CO2 is computed under the profile's own config; the default start time
        // is seeded after the presence is saved (see end of execute).
        let profile_settings = self.profile_settings.load(profile_id).await?;
        let default_start = profile_settings.default_start_minutes;
        let settings = &profile_settings.co2;

        // CO2 is tied to presence: only office/remote days carry a commute.
        let trips: Vec<TripInput> = if matches!(kind, PresenceType::Office | PresenceType::Remote) {
            trips
                .into_iter()
                .map(|t| t.into_domain(settings.default_car_occupancy))
                .collect()
        } else {
            Vec::new()
        };
        for (i, t) in trips.iter().enumerate() {
            if t.distance_km < 0.0 || !t.distance_km.is_finite() {
                return Err(DomainError::Validation(format!(
                    "trip {i}: distance must be >= 0"
                )));
            }
        }

        let (factor_map, variant_map) =
            load_factor_maps(&self.factors, settings.factor_year).await?;

        let day_em = Co2Calculator::compute_day(&trips, kind, &factor_map, &variant_map, settings);

        // Office/remote days store a (possibly zero) total; other types stay NULL
        // so the calendar shows no footprint for them.
        let co2_kg = if matches!(kind, PresenceType::Office | PresenceType::Remote) {
            Some(day_em.total_kg)
        } else {
            None
        };

        let now = self.clock.now_ms();
        let presence = Presence {
            id: Uuid::now_v7().to_string(),
            profile_id: profile_id.to_string(),
            day,
            kind,
            co2_kg,
            is_estimated: day_em.is_estimated,
            created_at: now,
            updated_at: now,
            work_minutes: 0,
        };

        // The persisted presence id is assigned by the repository (it may reuse
        // an existing row on conflict); leave `presence_id` empty here.
        let domain_trips: Vec<Trip> = day_em
            .trips
            .iter()
            .enumerate()
            .map(|(i, ct)| Trip {
                id: Uuid::now_v7().to_string(),
                mode_id: ct.mode_id.clone(),
                distance_km: ct.distance_km,
                round_trip: ct.round_trip,
                occupants: ct.occupants,
                co2_kg: ct.co2_kg,
                is_estimated: ct.is_estimated,
                factor_year: settings.factor_year,
                position: i as i64,
            })
            .collect();

        let saved = self.presences.set_for_day(&presence, &domain_trips).await?;

        // Seed the default start time once, the first time the day becomes a
        // work day. The guard is keyed on the PERSISTED id (set_for_day reuses
        // the existing row on conflict) and on "no schedule yet", so a user-set
        // start is never clobbered, while a re-created work day (e.g.
        // vacation -> office, whose type-change trigger dropped the old
        // schedule) gets re-seeded. A fresh day has no entries, so end == start.
        if matches!(saved.kind, PresenceType::Office | PresenceType::Remote)
            && self.entries.get_schedule(&saved.id).await?.is_none()
        {
            self.entries
                .set_schedule(&WorkDaySchedule {
                    presence_id: saved.id.clone(),
                    start_minutes: default_start,
                    end_minutes: default_start,
                })
                .await?;
        }

        Ok(PresenceDto::from(saved))
    }
}
