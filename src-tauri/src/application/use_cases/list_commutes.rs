use std::sync::Arc;

use crate::application::dto::commute_dto::CommuteDto;
use crate::application::use_cases::factor_maps::load_factor_maps;
use crate::domain::entities::presence::PresenceType;
use crate::domain::entities::trip::TripInput;
use crate::domain::error::DomainError;
use crate::domain::repositories::co2_settings_repository::Co2SettingsRepository;
use crate::domain::repositories::commute_repository::CommuteRepository;
use crate::domain::repositories::emission_factor_repository::EmissionFactorRepository;
use crate::domain::services::co2_calculator::Co2Calculator;

/// List a profile's saved commutes, each annotated with an indicative per-trip
/// CO2 footprint computed under the current settings (building energy excluded
/// from the preview so it reflects the commute alone).
pub struct ListCommutesUseCase {
    commutes: Arc<dyn CommuteRepository>,
    factors: Arc<dyn EmissionFactorRepository>,
    settings: Arc<dyn Co2SettingsRepository>,
}

impl ListCommutesUseCase {
    pub fn new(
        commutes: Arc<dyn CommuteRepository>,
        factors: Arc<dyn EmissionFactorRepository>,
        settings: Arc<dyn Co2SettingsRepository>,
    ) -> Self {
        Self {
            commutes,
            factors,
            settings,
        }
    }

    pub async fn execute(&self, profile_id: &str) -> Result<Vec<CommuteDto>, DomainError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(DomainError::Validation("profileId is required".to_string()));
        }

        let mut settings = self.settings.load().await?;
        // The preview is the commute's own footprint; never fold in building energy.
        settings.count_building_energy = false;

        let (factor_map, variant_map) =
            load_factor_maps(&self.factors, settings.factor_year).await?;

        let commutes = self.commutes.list_by_profile(profile_id).await?;
        let mut out = Vec::with_capacity(commutes.len());
        for c in commutes {
            let trips: Vec<TripInput> = c
                .segments
                .iter()
                .map(|s| TripInput {
                    mode_id: s.mode_id.clone(),
                    distance_km: s.distance_km,
                    round_trip: c.round_trip,
                    occupants: s.occupants,
                })
                .collect();
            let day = Co2Calculator::compute_day(
                &trips,
                PresenceType::Office,
                &factor_map,
                &variant_map,
                &settings,
            );
            let mut dto = CommuteDto::from(c);
            dto.co2_kg = Some(day.total_kg);
            out.push(dto);
        }
        Ok(out)
    }
}
