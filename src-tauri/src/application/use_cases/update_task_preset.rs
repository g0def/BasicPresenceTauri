use std::sync::Arc;

use crate::application::dto::task_preset_dto::TaskPresetDto;
use crate::application::use_cases::create_task_preset::{
    normalize_description, validate_color, validate_minutes, validate_title,
};
use crate::domain::error::DomainError;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::domain::services::clock::Clock;

/// Update a saved task preset. Past days are unaffected: day entries snapshot
/// the preset's fields when added.
pub struct UpdateTaskPresetUseCase {
    presets: Arc<dyn TaskPresetRepository>,
    clock: Arc<dyn Clock>,
}

impl UpdateTaskPresetUseCase {
    pub fn new(presets: Arc<dyn TaskPresetRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { presets, clock }
    }

    pub async fn execute(
        &self,
        id: &str,
        title: &str,
        description: Option<String>,
        default_minutes: i64,
        color: &str,
    ) -> Result<TaskPresetDto, DomainError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(DomainError::Validation("id is required".to_string()));
        }
        let mut preset = self
            .presets
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::Validation("task preset not found".to_string()))?;

        preset.title = validate_title(title)?;
        preset.description = normalize_description(description);
        preset.default_minutes = validate_minutes(default_minutes)?;
        preset.color = validate_color(color)?;
        preset.updated_at = self.clock.now_ms();

        self.presets.update(&preset).await?;
        Ok(TaskPresetDto::from(preset))
    }
}
