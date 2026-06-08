use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::repositories::commute_repository::CommuteRepository;

/// Delete a saved commute by id (its segments cascade).
pub struct DeleteCommuteUseCase {
    commutes: Arc<dyn CommuteRepository>,
}

impl DeleteCommuteUseCase {
    pub fn new(commutes: Arc<dyn CommuteRepository>) -> Self {
        Self { commutes }
    }

    pub async fn execute(&self, id: &str) -> Result<(), DomainError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(DomainError::Validation("id is required".to_string()));
        }
        self.commutes.delete(id).await
    }
}
