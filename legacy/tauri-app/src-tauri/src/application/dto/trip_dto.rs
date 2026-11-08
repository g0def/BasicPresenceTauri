use serde::{Deserialize, Serialize};

use crate::domain::entities::trip::{Trip, TripInput};
use crate::domain::error::DomainError;
use crate::domain::rules;

/// One trip leg received FROM the frontend when setting an office/remote day.
/// `occupants` falls back to the configured default when absent.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripInputDto {
    pub mode_id: String,
    pub distance_km: f64,
    pub round_trip: bool,
    pub occupants: Option<i64>,
}

impl TripInputDto {
    /// Validate (bounded distance/occupants, known-shape mode id) and convert.
    pub fn into_domain(
        self,
        position: usize,
        default_occupancy: i64,
    ) -> Result<TripInput, DomainError> {
        Ok(TripInput {
            mode_id: rules::mode_id(position, &self.mode_id)?,
            distance_km: rules::distance_km(position, self.distance_km)?,
            round_trip: self.round_trip,
            occupants: rules::occupants(
                position,
                Some(self.occupants.unwrap_or(default_occupancy)),
            )?,
        })
    }
}

/// A persisted trip snapshot returned to the frontend (for re-editing a day).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TripDto {
    pub id: String,
    pub mode_id: String,
    pub distance_km: f64,
    pub round_trip: bool,
    pub occupants: i64,
    pub co2_kg: f64,
    pub is_estimated: bool,
    pub position: i64,
}

impl From<Trip> for TripDto {
    fn from(t: Trip) -> Self {
        Self {
            id: t.id,
            mode_id: t.mode_id,
            distance_km: t.distance_km,
            round_trip: t.round_trip,
            occupants: t.occupants,
            co2_kg: t.co2_kg,
            is_estimated: t.is_estimated,
            position: t.position,
        }
    }
}
