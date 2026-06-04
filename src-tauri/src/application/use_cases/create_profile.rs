use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::profile_dto::ProfileDto;
use crate::domain::entities::profile::Profile;
use crate::domain::error::DomainError;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::domain::services::clock::Clock;

const MAX_FIELD_LEN: usize = 120;

/// Create a presence profile. The first profile created becomes the active one.
pub struct CreateProfileUseCase {
    profiles: Arc<dyn ProfileRepository>,
    clock: Arc<dyn Clock>,
}

impl CreateProfileUseCase {
    pub fn new(profiles: Arc<dyn ProfileRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { profiles, clock }
    }

    pub async fn execute(
        &self,
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

        let now = self.clock.now_ms();
        let profile = Profile {
            id: Uuid::now_v7().to_string(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            enterprise: enterprise.to_string(),
            poste,
            created_at: now,
            updated_at: now,
        };

        self.profiles.create(&profile).await?;
        // The first profile becomes the active one.
        if self.profiles.active_id().await?.is_none() {
            self.profiles.set_active(Some(&profile.id)).await?;
        }

        Ok(ProfileDto::from(profile))
    }
}

/// Validate a required, trimmed text field (non-empty, bounded length).
pub(crate) fn validate_required(field: &str, value: &str) -> Result<(), DomainError> {
    if value.is_empty() {
        return Err(DomainError::Validation(format!("{field} is required")));
    }
    if value.chars().count() > MAX_FIELD_LEN {
        return Err(DomainError::Validation(format!(
            "{field} must be at most {MAX_FIELD_LEN} characters"
        )));
    }
    Ok(())
}

/// Trim an optional field; an empty value becomes `None`. Bounds the length.
pub(crate) fn normalize_optional(
    field: &str,
    value: Option<&str>,
) -> Result<Option<String>, DomainError> {
    match value.map(str::trim).filter(|v| !v.is_empty()) {
        Some(v) if v.chars().count() > MAX_FIELD_LEN => Err(DomainError::Validation(format!(
            "{field} must be at most {MAX_FIELD_LEN} characters"
        ))),
        Some(v) => Ok(Some(v.to_string())),
        None => Ok(None),
    }
}
