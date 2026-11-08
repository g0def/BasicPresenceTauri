use crate::application::services::commute_builder::SegmentInput;
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

impl CommuteDto {
    /// Flatten the commute into the trip legs sent to `set_presence` when
    /// encoding an office/remote day: the journey-level round-trip flag is
    /// applied to every leg.
    pub fn to_trips(&self) -> Vec<crate::application::dto::trip_dto::TripInputDto> {
        self.segments
            .iter()
            .map(|s| crate::application::dto::trip_dto::TripInputDto {
                mode_id: s.mode_id.clone(),
                distance_km: s.distance_km,
                round_trip: self.round_trip,
                occupants: Some(s.occupants),
            })
            .collect()
    }
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

impl CommuteSegmentInputDto {
    pub fn as_segment_input(&self) -> SegmentInput<'_> {
        SegmentInput {
            mode_id: &self.mode_id,
            distance_km: self.distance_km,
            occupants: self.occupants,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_trips_applies_the_round_trip_flag_to_every_leg() {
        let segment = |mode: &str, km: f64, occupants: i64| CommuteSegmentDto {
            id: mode.into(),
            mode_id: mode.into(),
            distance_km: km,
            occupants,
            position: 0,
        };
        let commute = CommuteDto {
            id: "c".into(),
            profile_id: "p".into(),
            name: "Office".into(),
            round_trip: true,
            segments: vec![
                segment("car_petrol", 12.5, 2),
                segment("train_sncb", 40.0, 1),
            ],
            co2_kg: None,
            created_at: 0,
            updated_at: 0,
        };
        let trips = commute.to_trips();
        assert_eq!(trips.len(), 2);
        assert!(trips.iter().all(|t| t.round_trip));
        assert_eq!(trips[0].mode_id, "car_petrol");
        assert_eq!(trips[0].distance_km, 12.5);
        assert_eq!(trips[0].occupants, Some(2));
        assert_eq!(trips[1].mode_id, "train_sncb");
    }
}
