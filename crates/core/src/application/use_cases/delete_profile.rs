use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::repositories::profile_repository::ProfileRepository;

/// Delete a profile. If it was the active one, the active selection falls back
/// to the first remaining profile (or is cleared when none remain).
pub struct DeleteProfileUseCase {
    profiles: Arc<dyn ProfileRepository>,
}

impl DeleteProfileUseCase {
    pub fn new(profiles: Arc<dyn ProfileRepository>) -> Self {
        Self { profiles }
    }

    pub async fn execute(&self, id: &str) -> Result<(), DomainError> {
        let was_active = self.profiles.active_id().await?.as_deref() == Some(id);
        self.profiles.delete(id).await?;

        if was_active {
            let next = self.profiles.list().await?.first().map(|p| p.id.clone());
            self.profiles.set_active(next.as_deref()).await?;
        }
        Ok(())
    }
}
