use std::sync::Arc;

use crate::application::dto::task_preset_dto::TaskPresetDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::domain::rules;
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
        let id = rules::required_id("id", id)?;
        let mut preset = self
            .presets
            .find_by_id(id)
            .await?
            .ok_or_else(|| DomainError::Validation("task preset not found".to_string()))?;

        preset.title = rules::title(title)?;
        preset.description = rules::description(description.as_deref())?;
        preset.default_minutes = rules::task_minutes(default_minutes)?;
        preset.color = rules::color(color)?;
        preset.updated_at = self.clock.now_ms();

        self.presets.update(&preset).await?;
        Ok(TaskPresetDto::from(preset))
    }
}
