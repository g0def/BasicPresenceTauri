use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;

/// Delete a saved task preset. Day entries that were added from it keep their
/// snapshotted title/color.
pub struct DeleteTaskPresetUseCase {
    presets: Arc<dyn TaskPresetRepository>,
}

impl DeleteTaskPresetUseCase {
    pub fn new(presets: Arc<dyn TaskPresetRepository>) -> Self {
        Self { presets }
    }

    pub async fn execute(&self, id: &str) -> Result<(), DomainError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(DomainError::Validation("id is required".to_string()));
        }
        self.presets.delete(id).await
    }
}
