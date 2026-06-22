use serde::Serialize;

/// One per-country electric grid override, exposed to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GridVariantDto {
    pub country: String,
    pub value: f64,
}

/// An emission factor as offered to the frontend mode pickers and the methodology
/// page. `category` drives UI grouping; `gridVariants` lists per-country overrides
/// for electric modes; `scope`/`source` are traceability metadata (null when the
/// referential row leaves them blank).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionFactorDto {
    pub mode_id: String,
    pub label: String,
    pub value: f64,
    pub unit: String,
    pub category: String,
    pub is_param: bool,
    pub grid_variants: Vec<GridVariantDto>,
    pub scope: Option<String>,
    pub source: Option<String>,
}

/// The full CO2 referential for the methodology page: every factor (including the
/// `building` rows hidden from the trip pickers) plus the active referential year
/// and the radiative-forcing multiplier in force for that year.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Co2ReferentialDto {
    pub factor_year: i32,
    pub radiative_forcing: f64,
    pub factors: Vec<EmissionFactorDto>,
}
