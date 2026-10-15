use std::sync::Arc;

use crate::application::dto::profile_dto::ProfileDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::domain::rules;
use crate::domain::services::clock::Clock;

/// Update an existing profile's fields (preserving its `created_at`).
pub struct UpdateProfileUseCase {
    profiles: Arc<dyn ProfileRepository>,
    clock: Arc<dyn Clock>,
}

impl UpdateProfileUseCase {
    pub fn new(profiles: Arc<dyn ProfileRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { profiles, clock }
    }

    pub async fn execute(
        &self,
        id: &str,
        first_name: &str,
        last_name: &str,
        enterprise: &str,
        poste: Option<&str>,
    ) -> Result<ProfileDto, DomainError> {
        let first_name = rules::profile_field("firstName", first_name)?;
        let last_name = rules::profile_field("lastName", last_name)?;
        let enterprise = rules::profile_field("enterprise", enterprise)?;
        let poste = rules::profile_optional_field("poste", poste)?;

        let id = rules::required_id("id", id)?;
        let Some(mut profile) = self.profiles.find_by_id(id).await? else {
            return Err(DomainError::ProfileNotFound);
        };

        profile.first_name = first_name;
        profile.last_name = last_name;
        profile.enterprise = enterprise;
        profile.poste = poste;
        profile.updated_at = self.clock.now_ms();

        self.profiles.update(&profile).await?;
        Ok(ProfileDto::from(profile))
    }
}
