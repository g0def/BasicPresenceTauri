use async_trait::async_trait;

use crate::domain::entities::profile::Profile;
use crate::domain::error::DomainError;

/// Persistence contract for presence profiles (implemented over the unlocked
/// vault connection). The "active" profile id is tracked in `vault_meta`.
#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn create(&self, profile: &Profile) -> Result<(), DomainError>;

    async fn list(&self) -> Result<Vec<Profile>, DomainError>;

    async fn find_by_id(&self, id: &str) -> Result<Option<Profile>, DomainError>;

    async fn update(&self, profile: &Profile) -> Result<(), DomainError>;

    async fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// Id of the currently active profile, if one is set.
    async fn active_id(&self) -> Result<Option<String>, DomainError>;

    /// Set (or clear, with `None`) the active profile id.
    async fn set_active(&self, id: Option<&str>) -> Result<(), DomainError>;
}
