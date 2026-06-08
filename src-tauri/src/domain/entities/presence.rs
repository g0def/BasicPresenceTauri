use crate::domain::error::DomainError;

/// The kind of a day's presence. Stored as a lowercase string in SQL; the
/// domain owns the (de)serialization and the validation of unknown values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceType {
    Office,
    Remote,
    Vacation,
    Holiday,
}

impl PresenceType {
    /// Lowercase wire/storage representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            PresenceType::Office => "office",
            PresenceType::Remote => "remote",
            PresenceType::Vacation => "vacation",
            PresenceType::Holiday => "holiday",
        }
    }

    /// Parse a stored/incoming string into a `PresenceType`, rejecting unknowns.
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        match value {
            "office" => Ok(PresenceType::Office),
            "remote" => Ok(PresenceType::Remote),
            "vacation" => Ok(PresenceType::Vacation),
            "holiday" => Ok(PresenceType::Holiday),
            other => Err(DomainError::Validation(format!(
                "invalid presence type: {other}"
            ))),
        }
    }
}

/// A single day of presence for a profile (lives in the encrypted vault).
#[derive(Debug, Clone)]
pub struct Presence {
    pub id: String,
    pub profile_id: String,
    /// Epoch ms at UTC midnight of the day concerned.
    pub day: i64,
    pub kind: PresenceType,
    /// Denormalized commute footprint for the day (kgCO2e). `None` when the day
    /// carries no trip (e.g. vacation/holiday, or a pre-CO2 imported row).
    pub co2_kg: Option<f64>,
    /// `true` if any of the day's trips fell back to an estimated factor (R6).
    pub is_estimated: bool,
    pub created_at: i64,
    pub updated_at: i64,
}
