use std::sync::Arc;

use crate::application::dto::profile_dto::ProfileDto;
use crate::application::use_cases::create_profile::{normalize_optional, validate_required};
use crate::domain::error::DomainError;
use crate::domain::repositories::profile_repository::ProfileRepository;
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
        let first_name = first_name.trim();
        let last_name = last_name.trim();
        let enterprise = enterprise.trim();
        validate_required("firstName", first_name)?;
        validate_required("lastName", last_name)?;
        validate_required("enterprise", enterprise)?;
        let poste = normalize_optional("poste", poste)?;

        let Some(mut profile) = self.profiles.find_by_id(id).await? else {
            return Err(DomainError::ProfileNotFound);
        };

        profile.first_name = first_name.to_string();
        profile.last_name = last_name.to_string();
        profile.enterprise = enterprise.to_string();
        profile.poste = poste;
        profile.updated_at = self.clock.now_ms();

        self.profiles.update(&profile).await?;
        Ok(ProfileDto::from(profile))
    }
}
