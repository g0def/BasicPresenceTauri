use serde::{Deserialize, Serialize};

use crate::domain::entities::trip::{Trip, TripInput};

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
    pub fn into_domain(self, default_occupancy: i64) -> TripInput {
        TripInput {
            mode_id: self.mode_id,
            distance_km: self.distance_km,
            round_trip: self.round_trip,
            occupants: self.occupants.unwrap_or(default_occupancy).max(1),
        }
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
