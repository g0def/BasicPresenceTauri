use async_trait::async_trait;

use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::error::DomainError;

/// Per-profile settings persistence (over the unlocked vault connection).
/// `load` returns [`ProfileSettings::default`] when the profile has no row
/// (lazy default); `save` upserts the whole row.
#[async_trait]
pub trait ProfileSettingsRepository: Send + Sync {
    async fn load(&self, profile_id: &str) -> Result<ProfileSettings, DomainError>;

    async fn save(&self, profile_id: &str, settings: &ProfileSettings) -> Result<(), DomainError>;
}
