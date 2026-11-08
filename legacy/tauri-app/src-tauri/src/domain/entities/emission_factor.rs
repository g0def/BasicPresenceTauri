use crate::domain::error::DomainError;

/// The unit a factor's `value` is expressed in. Drives the per-trip arithmetic:
/// only `VehKm` (per-vehicle) factors are divided by the number of occupants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmissionUnit {
    /// kgCO2e per vehicle-kilometre (split across occupants for carpool).
    VehKm,
    /// kgCO2e per passenger-kilometre (already per person).
    PassengerKm,
    /// kgCO2e per kilometre (absolute, e.g. active/micro-mobility).
    Km,
    /// kgCO2e per day (building energy).
    Day,
}

impl EmissionUnit {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmissionUnit::VehKm => "kgCO2e/veh.km",
            EmissionUnit::PassengerKm => "kgCO2e/passenger.km",
            EmissionUnit::Km => "kgCO2e/km",
            EmissionUnit::Day => "kgCO2e/day",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "kgCO2e/veh.km" => Ok(EmissionUnit::VehKm),
            "kgCO2e/passenger.km" => Ok(EmissionUnit::PassengerKm),
            "kgCO2e/km" => Ok(EmissionUnit::Km),
            "kgCO2e/day" => Ok(EmissionUnit::Day),
            other => Err(DomainError::Validation(format!(
                "invalid emission unit: {other}"
            ))),
        }
    }
}

/// Coarse grouping of a transport mode. Used for UI grouping and to gate the
/// aviation radiative-forcing adjustment (applied only to `Air`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmissionCategory {
    Car,
    Active,
    PublicTransport,
    Rail,
    Air,
    Building,
}

impl EmissionCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmissionCategory::Car => "car",
            EmissionCategory::Active => "active",
            EmissionCategory::PublicTransport => "public_transport",
            EmissionCategory::Rail => "rail",
            EmissionCategory::Air => "air",
            EmissionCategory::Building => "building",
        }
    }

    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "car" => Ok(EmissionCategory::Car),
            "active" => Ok(EmissionCategory::Active),
            "public_transport" => Ok(EmissionCategory::PublicTransport),
            "rail" => Ok(EmissionCategory::Rail),
            "air" => Ok(EmissionCategory::Air),
            "building" => Ok(EmissionCategory::Building),
            other => Err(DomainError::Validation(format!(
                "invalid emission category: {other}"
            ))),
        }
    }
}

/// One row of the versioned emission-factor referential (lives in the vault).
/// The reference `year` column drives versioning; `scope` and `source` are
/// traceability metadata surfaced on the methodology page.
#[derive(Debug, Clone)]
pub struct EmissionFactor {
    /// The mode_id, e.g. `"car_petrol"`.
    pub id: String,
    pub label: String,
    pub value: f64,
    pub unit: EmissionUnit,
    pub category: EmissionCategory,
    /// `true` when the value can be overridden by a parameter (the grid country).
    pub is_param: bool,
    /// Perimeter of the factor (e.g. `"usage+fabrication"`), for traceability.
    pub scope: Option<String>,
    /// Source attribution (e.g. `"ADEME Base Carbone"`), for traceability.
    pub source: Option<String>,
}

/// A per-(mode, country) override of an electric mode's factor.
#[derive(Debug, Clone)]
pub struct GridVariant {
    pub mode_id: String,
    pub country: String,
    pub value: f64,
}
