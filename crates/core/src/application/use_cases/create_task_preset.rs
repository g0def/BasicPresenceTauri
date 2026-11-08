use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::task_preset_dto::TaskPresetDto;
use crate::domain::entities::task_preset::TaskPreset;
use crate::domain::error::DomainError;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::domain::rules;
use crate::domain::services::clock::Clock;

/// Create a saved task preset (title, default duration, color) for a profile.
pub struct CreateTaskPresetUseCase {
    presets: Arc<dyn TaskPresetRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateTaskPresetUseCase {
    pub fn new(presets: Arc<dyn TaskPresetRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { presets, clock }
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        title: &str,
        description: Option<String>,
        default_minutes: i64,
        color: &str,
    ) -> Result<TaskPresetDto, DomainError> {
        let profile_id = rules::required_id("profileId", profile_id)?;
        let now = self.clock.now_ms();
        let preset = TaskPreset {
            id: Uuid::now_v7().to_string(),
            profile_id: profile_id.to_string(),
            title: rules::title(title)?,
            description: rules::description(description.as_deref())?,
            default_minutes: rules::task_minutes(default_minutes)?,
            color: rules::color(color)?,
            created_at: now,
            updated_at: now,
        };
        self.presets.create(&preset).await?;
        Ok(TaskPresetDto::from(preset))
    }
}
