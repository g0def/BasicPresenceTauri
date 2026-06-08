use std::sync::Arc;

use crate::application::dto::trip_dto::TripDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;

/// Fetch the trip snapshot of a presence day, to pre-fill the picker when a
/// previously-encoded office/remote day is reopened for editing.
pub struct GetPresenceTripsUseCase {
    presences: Arc<dyn PresenceRepository>,
}

impl GetPresenceTripsUseCase {
    pub fn new(presences: Arc<dyn PresenceRepository>) -> Self {
        Self { presences }
    }

    pub async fn execute(&self, presence_id: &str) -> Result<Vec<TripDto>, DomainError> {
        let presence_id = presence_id.trim();
        if presence_id.is_empty() {
            return Err(DomainError::Validation(
                "presenceId is required".to_string(),
            ));
        }
        let trips = self.presences.list_trips(presence_id).await?;
        Ok(trips.into_iter().map(TripDto::from).collect())
    }
}
