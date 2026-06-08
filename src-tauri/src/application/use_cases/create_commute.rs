use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::commute_dto::{CommuteDto, CommuteSegmentInputDto};
use crate::domain::entities::commute::{Commute, CommuteSegment};
use crate::domain::error::DomainError;
use crate::domain::repositories::commute_repository::CommuteRepository;
use crate::domain::services::clock::Clock;

/// Create a saved commute template (name + ordered segments) for a profile.
pub struct CreateCommuteUseCase {
    commutes: Arc<dyn CommuteRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateCommuteUseCase {
    pub fn new(commutes: Arc<dyn CommuteRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { commutes, clock }
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        name: &str,
        round_trip: bool,
        segments: Vec<CommuteSegmentInputDto>,
    ) -> Result<CommuteDto, DomainError> {
        let commute = build_commute(
            Uuid::now_v7().to_string(),
            profile_id,
            name,
            round_trip,
            segments,
            self.clock.now_ms(),
            self.clock.now_ms(),
        )?;
        self.commutes.create(&commute).await?;
        Ok(CommuteDto::from(commute))
    }
}

/// Validate inputs and assemble a `Commute` with fresh segment ids/positions.
/// Shared by create (fresh id/timestamps) and update (existing id/created_at).
pub fn build_commute(
    id: String,
    profile_id: &str,
    name: &str,
    round_trip: bool,
    segments: Vec<CommuteSegmentInputDto>,
    created_at: i64,
    updated_at: i64,
) -> Result<Commute, DomainError> {
    let profile_id = profile_id.trim();
    if profile_id.is_empty() {
        return Err(DomainError::Validation("profileId is required".to_string()));
    }
    let name = name.trim();
    if name.is_empty() {
        return Err(DomainError::Validation("name is required".to_string()));
    }
    if segments.is_empty() {
        return Err(DomainError::Validation(
            "at least one segment is required".to_string(),
        ));
    }

    let mut domain_segments = Vec::with_capacity(segments.len());
    for (i, s) in segments.into_iter().enumerate() {
        let mode_id = s.mode_id.trim();
        if mode_id.is_empty() {
            return Err(DomainError::Validation(format!(
                "segment {i}: mode is required"
            )));
        }
        if s.distance_km < 0.0 || !s.distance_km.is_finite() {
            return Err(DomainError::Validation(format!(
                "segment {i}: distance must be >= 0"
            )));
        }
        domain_segments.push(CommuteSegment {
            id: Uuid::now_v7().to_string(),
            commute_id: id.clone(),
            mode_id: mode_id.to_string(),
            distance_km: s.distance_km,
            occupants: s.occupants.unwrap_or(1).max(1),
            position: i as i64,
        });
    }

    Ok(Commute {
        id,
        profile_id: profile_id.to_string(),
        name: name.to_string(),
        round_trip,
        segments: domain_segments,
        created_at,
        updated_at,
    })
}
