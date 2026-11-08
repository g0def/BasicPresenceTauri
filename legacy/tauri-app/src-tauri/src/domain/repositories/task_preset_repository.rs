use async_trait::async_trait;

use crate::domain::entities::task_preset::TaskPreset;
use crate::domain::error::DomainError;

/// Persistence contract for saved task presets (implemented over the unlocked
/// vault connection).
#[async_trait]
pub trait TaskPresetRepository: Send + Sync {
    async fn create(&self, preset: &TaskPreset) -> Result<(), DomainError>;

    async fn list_by_profile(&self, profile_id: &str) -> Result<Vec<TaskPreset>, DomainError>;

    async fn find_by_id(&self, id: &str) -> Result<Option<TaskPreset>, DomainError>;

    async fn update(&self, preset: &TaskPreset) -> Result<(), DomainError>;

    async fn delete(&self, id: &str) -> Result<(), DomainError>;
}
