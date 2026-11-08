use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::repositories::profile_repository::ProfileRepository;

/// Mark a profile as the active one (validates it exists first).
pub struct SetActiveProfileUseCase {
    profiles: Arc<dyn ProfileRepository>,
}

impl SetActiveProfileUseCase {
    pub fn new(profiles: Arc<dyn ProfileRepository>) -> Self {
        Self { profiles }
    }

    pub async fn execute(&self, id: &str) -> Result<(), DomainError> {
        if self.profiles.find_by_id(id).await?.is_none() {
            return Err(DomainError::ProfileNotFound);
        }
        self.profiles.set_active(Some(id)).await
    }
}
