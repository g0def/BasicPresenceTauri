use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::commute_dto::{CommuteDto, CommuteSegmentInputDto};
use crate::application::services::commute_builder::build_commute;
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
            segments
                .iter()
                .map(CommuteSegmentInputDto::as_segment_input),
            self.clock.now_ms(),
            self.clock.now_ms(),
        )?;
        self.commutes.create(&commute).await?;
        Ok(CommuteDto::from(commute))
    }
}
