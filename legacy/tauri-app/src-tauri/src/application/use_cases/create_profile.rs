use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::profile_dto::ProfileDto;
use crate::domain::entities::profile::Profile;
use crate::domain::error::DomainError;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::domain::rules;
use crate::domain::services::clock::Clock;
use crate::domain::services::unit_of_work::UnitOfWork;

/// Create a presence profile. The first profile created becomes the active one.
pub struct CreateProfileUseCase {
    profiles: Arc<dyn ProfileRepository>,
    clock: Arc<dyn Clock>,
    uow: Arc<dyn UnitOfWork>,
}

impl CreateProfileUseCase {
    pub fn new(
        profiles: Arc<dyn ProfileRepository>,
        clock: Arc<dyn Clock>,
        uow: Arc<dyn UnitOfWork>,
    ) -> Self {
        Self {
            profiles,
            clock,
            uow,
        }
    }

    pub async fn execute(
        &self,
        first_name: &str,
        last_name: &str,
        enterprise: &str,
        poste: Option<&str>,
    ) -> Result<ProfileDto, DomainError> {
        let first_name = rules::profile_field("firstName", first_name)?;
        let last_name = rules::profile_field("lastName", last_name)?;
        let enterprise = rules::profile_field("enterprise", enterprise)?;
        let poste = rules::profile_optional_field("poste", poste)?;

        let now = self.clock.now_ms();
        let profile = Profile {
            id: Uuid::now_v7().to_string(),
            first_name,
            last_name,
            enterprise,
            poste,
            created_at: now,
            updated_at: now,
        };

        // The profile and (for the first one) the active-profile marker commit
        // together.
        self.uow
            .run(Box::pin(async {
                self.profiles.create(&profile).await?;
                // The first profile becomes the active one.
                if self.profiles.active_id().await?.is_none() {
                    self.profiles.set_active(Some(&profile.id)).await?;
                }
                Ok(())
            }))
            .await?;

        Ok(ProfileDto::from(profile))
    }
}
