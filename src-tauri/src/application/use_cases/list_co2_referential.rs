use std::collections::HashMap;
use std::sync::Arc;

use crate::application::dto::emission_factor_dto::{
    Co2ReferentialDto, EmissionFactorDto, GridVariantDto,
};
use crate::domain::entities::co2_settings::Co2Settings;
use crate::domain::error::DomainError;
use crate::domain::repositories::emission_factor_repository::EmissionFactorRepository;
use crate::domain::services::co2_calculator::radiative_forcing_factor;

/// List the FULL CO2 referential for the methodology page: every factor (unlike
/// the trip picker, the `building` rows are kept), each with its scope/source and
/// per-country grid variants, plus the active referential year and the
/// radiative-forcing multiplier in force for that year. Profile-agnostic: it
/// reports the default referential year, the one new days are computed against.
pub struct ListCo2ReferentialUseCase {
    factors: Arc<dyn EmissionFactorRepository>,
}

impl ListCo2ReferentialUseCase {
    pub fn new(factors: Arc<dyn EmissionFactorRepository>) -> Self {
        Self { factors }
    }

    pub async fn execute(&self) -> Result<Co2ReferentialDto, DomainError> {
        let factor_year = Co2Settings::default().factor_year;
        let factors = self.factors.list(factor_year).await?;
        let variants = self.factors.list_grid_variants(factor_year).await?;

        let mut by_mode: HashMap<String, Vec<GridVariantDto>> = HashMap::new();
        for v in variants {
            by_mode.entry(v.mode_id).or_default().push(GridVariantDto {
                country: v.country,
                value: v.value,
            });
        }

        let factors = factors
            .into_iter()
            .map(|f| {
                let grid_variants = by_mode.get(&f.id).cloned().unwrap_or_default();
                EmissionFactorDto {
                    mode_id: f.id,
                    label: f.label,
                    value: f.value,
                    unit: f.unit.as_str().to_string(),
                    category: f.category.as_str().to_string(),
                    is_param: f.is_param,
                    grid_variants,
                    scope: f.scope,
                    source: f.source,
                }
            })
            .collect();

        Ok(Co2ReferentialDto {
            factor_year,
            radiative_forcing: radiative_forcing_factor(factor_year),
            factors,
        })
    }
}
