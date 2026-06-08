use serde::Serialize;

/// One per-country electric grid override, exposed to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GridVariantDto {
    pub country: String,
    pub value: f64,
}

/// An emission factor as offered to the frontend mode pickers. `category` drives
/// UI grouping; `gridVariants` lists per-country overrides for electric modes.
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
}
