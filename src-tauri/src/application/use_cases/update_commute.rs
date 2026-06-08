use std::sync::Arc;

use crate::application::dto::commute_dto::{CommuteDto, CommuteSegmentInputDto};
use crate::application::use_cases::create_commute::build_commute;
use crate::domain::error::DomainError;
use crate::domain::repositories::commute_repository::CommuteRepository;
use crate::domain::services::clock::Clock;

/// Update a saved commute (name, round-trip flag, and full segment set).
pub struct UpdateCommuteUseCase {
    commutes: Arc<dyn CommuteRepository>,
    clock: Arc<dyn Clock>,
}

impl UpdateCommuteUseCase {
    pub fn new(commutes: Arc<dyn CommuteRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { commutes, clock }
    }

    pub async fn execute(
        &self,
        id: &str,
        name: &str,
        round_trip: bool,
        segments: Vec<CommuteSegmentInputDto>,
    ) -> Result<CommuteDto, DomainError> {
        let existing = self
            .commutes
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::Validation("commute not found".to_string()))?;

        let commute = build_commute(
            existing.id,
            &existing.profile_id,
            name,
            round_trip,
            segments,
            existing.created_at,
            self.clock.now_ms(),
        )?;
        self.commutes.update(&commute).await?;
        Ok(CommuteDto::from(commute))
    }
}
