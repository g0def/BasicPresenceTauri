use async_trait::async_trait;

use crate::domain::entities::presence::Presence;
use crate::domain::error::DomainError;

/// Persistence contract for daily presences (implemented over the unlocked
/// vault connection). At most one presence exists per `(profile_id, day)`.
#[async_trait]
pub trait PresenceRepository: Send + Sync {
    /// Create or update the presence for a `(profile_id, day)` and return the
    /// persisted row (preserving the original `id`/`created_at` on update).
    async fn set_for_day(&self, presence: &Presence) -> Result<Presence, DomainError>;

    async fn list_by_profile(&self, profile_id: &str) -> Result<Vec<Presence>, DomainError>;

    async fn delete(&self, id: &str) -> Result<(), DomainError>;
}
