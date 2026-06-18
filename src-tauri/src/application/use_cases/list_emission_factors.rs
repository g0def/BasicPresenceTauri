use std::collections::HashMap;
use std::sync::Arc;

use crate::application::dto::emission_factor_dto::{EmissionFactorDto, GridVariantDto};
use crate::domain::entities::co2_settings::Co2Settings;
use crate::domain::entities::emission_factor::EmissionCategory;
use crate::domain::error::DomainError;
use crate::domain::repositories::emission_factor_repository::EmissionFactorRepository;

/// List the emission-factor referential for the picker. The referential year is
/// fixed (only one year is seeded) and is not user-editable, so the picker is
/// profile-agnostic and always uses the default factor year.
pub struct ListEmissionFactorsUseCase {
    factors: Arc<dyn EmissionFactorRepository>,
}

impl ListEmissionFactorsUseCase {
    pub fn new(factors: Arc<dyn EmissionFactorRepository>) -> Self {
        Self { factors }
    }

    pub async fn execute(&self) -> Result<Vec<EmissionFactorDto>, DomainError> {
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

        Ok(factors
            .into_iter()
            .filter(|f| f.category != EmissionCategory::Building)
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
                }
            })
            .collect())
    }
}
