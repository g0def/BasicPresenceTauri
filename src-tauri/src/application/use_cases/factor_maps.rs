use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entities::emission_factor::EmissionFactor;
use crate::domain::error::DomainError;
use crate::domain::repositories::emission_factor_repository::EmissionFactorRepository;

pub(crate) type FactorMap = HashMap<String, EmissionFactor>;
pub(crate) type GridVariantMap = HashMap<(String, String), f64>;

/// Load a year's emission-factor referential as the lookup maps expected by
/// `Co2Calculator` (shared by the presence and commute use cases).
pub(crate) async fn load_factor_maps(
    factors: &Arc<dyn EmissionFactorRepository>,
    year: i32,
) -> Result<(FactorMap, GridVariantMap), DomainError> {
    let factor_map: FactorMap = factors
        .list(year)
        .await?
        .into_iter()
        .map(|f| (f.id.clone(), f))
        .collect();
    // An unseeded vintage must not silently resolve every trip to 0.0 kgCO2e:
    // `Co2Calculator` finds neither the mode nor the `car_average` fallback and
    // returns zero, which then gets snapshotted per day. A null carbon report is
    // the worst possible output for this tool, so fail loudly instead.
    if factor_map.is_empty() {
        return Err(DomainError::Validation(format!(
            "no emission factors are seeded for reference year {year}"
        )));
    }

    let variant_map: GridVariantMap = factors
        .list_grid_variants(year)
        .await?
        .into_iter()
        .map(|v| ((v.mode_id, v.country), v.value))
        .collect();
    Ok((factor_map, variant_map))
}
