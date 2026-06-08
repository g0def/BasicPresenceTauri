use serde::{Deserialize, Serialize};

use crate::domain::entities::commute::Commute;

/// A commute segment returned to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommuteSegmentDto {
    pub id: String,
    pub mode_id: String,
    pub distance_km: f64,
    pub occupants: i64,
    pub position: i64,
}

/// A saved commute template returned to the frontend. `co2Kg` is an indicative
/// per-day footprint computed with the current settings (set by `list_commutes`;
/// `None` on create/update responses).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommuteDto {
    pub id: String,
    pub profile_id: String,
    pub name: String,
    pub round_trip: bool,
    pub segments: Vec<CommuteSegmentDto>,
    pub co2_kg: Option<f64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Commute> for CommuteDto {
    fn from(c: Commute) -> Self {
        Self {
            id: c.id,
            profile_id: c.profile_id,
            name: c.name,
            round_trip: c.round_trip,
            segments: c
                .segments
                .into_iter()
                .map(|s| CommuteSegmentDto {
                    id: s.id,
                    mode_id: s.mode_id,
                    distance_km: s.distance_km,
                    occupants: s.occupants,
                    position: s.position,
                })
                .collect(),
            co2_kg: None,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

/// A commute segment received FROM the frontend on create/update. `occupants`
/// falls back to the configured default when absent.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommuteSegmentInputDto {
    pub mode_id: String,
    pub distance_km: f64,
    pub occupants: Option<i64>,
}
