use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;

/// Delete a single presence by id.
pub struct DeletePresenceUseCase {
    presences: Arc<dyn PresenceRepository>,
}

impl DeletePresenceUseCase {
    pub fn new(presences: Arc<dyn PresenceRepository>) -> Self {
        Self { presences }
    }

    pub async fn execute(&self, id: &str) -> Result<(), DomainError> {
        self.presences.delete(id).await
    }
}
