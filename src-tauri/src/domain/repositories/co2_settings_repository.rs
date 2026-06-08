use async_trait::async_trait;

use crate::domain::entities::co2_settings::Co2Settings;
use crate::domain::error::DomainError;

/// Loads the org-level CO2 configuration (from `vault_meta`), falling back to
/// defaults when none has been stored.
#[async_trait]
pub trait Co2SettingsRepository: Send + Sync {
    async fn load(&self) -> Result<Co2Settings, DomainError>;
}
