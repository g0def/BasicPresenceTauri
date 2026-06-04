use std::sync::Arc;

use crate::application::dto::profile_dto::{ProfileDto, ProfilesDto};
use crate::domain::error::DomainError;
use crate::domain::repositories::profile_repository::ProfileRepository;

/// List every profile plus the active profile id.
pub struct ListProfilesUseCase {
    profiles: Arc<dyn ProfileRepository>,
}

impl ListProfilesUseCase {
    pub fn new(profiles: Arc<dyn ProfileRepository>) -> Self {
        Self { profiles }
    }

    pub async fn execute(&self) -> Result<ProfilesDto, DomainError> {
        let profiles = self
            .profiles
            .list()
            .await?
            .into_iter()
            .map(ProfileDto::from)
            .collect();
        let active_profile_id = self.profiles.active_id().await?;
        Ok(ProfilesDto {
            profiles,
            active_profile_id,
        })
    }
}
