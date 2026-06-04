use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::presence_dto::PresenceDto;
use crate::domain::entities::presence::{Presence, PresenceType};
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::services::clock::Clock;

/// Milliseconds in a UTC day. A valid `day` is a non-negative multiple of it
/// (epoch ms at UTC midnight), matching the key the frontend sends.
const MS_PER_DAY: i64 = 86_400_000;

/// Set (create or update) the presence type for a given day of a profile.
/// Re-setting the same day overwrites the type rather than duplicating the row.
pub struct SetPresenceUseCase {
    presences: Arc<dyn PresenceRepository>,
    clock: Arc<dyn Clock>,
}

impl SetPresenceUseCase {
    pub fn new(presences: Arc<dyn PresenceRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { presences, clock }
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        day: i64,
        kind: &str,
    ) -> Result<PresenceDto, DomainError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(DomainError::Validation("profileId is required".to_string()));
        }
        if day < 0 || day % MS_PER_DAY != 0 {
            return Err(DomainError::Validation(
                "day must be epoch ms at UTC midnight".to_string(),
            ));
        }
        let kind = PresenceType::parse(kind)?;

        let now = self.clock.now_ms();
        let presence = Presence {
            id: Uuid::now_v7().to_string(),
            profile_id: profile_id.to_string(),
            day,
            kind,
            created_at: now,
            updated_at: now,
        };

        let saved = self.presences.set_for_day(&presence).await?;
        Ok(PresenceDto::from(saved))
    }
}
