use std::sync::Arc;

use crate::application::dto::presence_dto::PresenceDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;

/// List every presence recorded for a profile (ordered by day).
pub struct ListPresencesUseCase {
    presences: Arc<dyn PresenceRepository>,
}

impl ListPresencesUseCase {
    pub fn new(presences: Arc<dyn PresenceRepository>) -> Self {
        Self { presences }
    }

    pub async fn execute(&self, profile_id: &str) -> Result<Vec<PresenceDto>, DomainError> {
        let presences = self
            .presences
            .list_by_profile(profile_id)
            .await?
            .into_iter()
            .map(PresenceDto::from)
            .collect();
        Ok(presences)
    }
}
