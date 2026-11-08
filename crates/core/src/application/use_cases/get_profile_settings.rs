use std::sync::Arc;

use crate::application::dto::profile_settings_dto::ProfileSettingsDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::profile_settings_repository::ProfileSettingsRepository;

/// Load a profile's settings (lazy defaults when no row has been stored yet).
pub struct GetProfileSettingsUseCase {
    settings: Arc<dyn ProfileSettingsRepository>,
}

impl GetProfileSettingsUseCase {
    pub fn new(settings: Arc<dyn ProfileSettingsRepository>) -> Self {
        Self { settings }
    }

    pub async fn execute(&self, profile_id: &str) -> Result<ProfileSettingsDto, DomainError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(DomainError::Validation("profileId is required".to_string()));
        }
        Ok(ProfileSettingsDto::from(
            self.settings.load(profile_id).await?,
        ))
    }
}
