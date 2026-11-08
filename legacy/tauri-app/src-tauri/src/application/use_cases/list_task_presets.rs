use std::sync::Arc;

use crate::application::dto::task_preset_dto::TaskPresetDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;

/// List a profile's saved task presets (alphabetical).
pub struct ListTaskPresetsUseCase {
    presets: Arc<dyn TaskPresetRepository>,
}

impl ListTaskPresetsUseCase {
    pub fn new(presets: Arc<dyn TaskPresetRepository>) -> Self {
        Self { presets }
    }

    pub async fn execute(&self, profile_id: &str) -> Result<Vec<TaskPresetDto>, DomainError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(DomainError::Validation("profileId is required".to_string()));
        }
        let presets = self.presets.list_by_profile(profile_id).await?;
        Ok(presets.into_iter().map(TaskPresetDto::from).collect())
    }
}
