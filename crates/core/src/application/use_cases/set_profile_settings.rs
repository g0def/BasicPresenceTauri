use std::sync::Arc;

use crate::application::dto::profile_settings_dto::ProfileSettingsDto;
use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::error::DomainError;
use crate::domain::repositories::profile_settings_repository::ProfileSettingsRepository;
use crate::domain::rules;

/// Persist a profile's full settings row (the frontend merges partial edits onto
/// the current row before sending the canonical whole).
pub struct SetProfileSettingsUseCase {
    settings: Arc<dyn ProfileSettingsRepository>,
}

impl SetProfileSettingsUseCase {
    pub fn new(settings: Arc<dyn ProfileSettingsRepository>) -> Self {
        Self { settings }
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        dto: ProfileSettingsDto,
    ) -> Result<ProfileSettingsDto, DomainError> {
        let profile_id = rules::required_id("profileId", profile_id)?;
        let settings = ProfileSettings::from(dto);
        rules::profile_settings(&settings)?;
        self.settings.save(profile_id, &settings).await?;
        Ok(ProfileSettingsDto::from(settings))
    }
}
