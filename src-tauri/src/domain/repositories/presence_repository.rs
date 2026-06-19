use async_trait::async_trait;

use crate::domain::entities::presence::Presence;
use crate::domain::entities::trip::Trip;
use crate::domain::error::DomainError;

/// Persistence contract for daily presences (implemented over the unlocked
/// vault connection). At most one presence exists per `(profile_id, day)`.
#[async_trait]
pub trait PresenceRepository: Send + Sync {
    /// Create or update the presence for a `(profile_id, day)` and atomically
    /// replace its commute trip snapshot, returning the persisted row
    /// (preserving the original `id`/`created_at` on update). Passing an empty
    /// `trips` clears the day's trips.
    async fn set_for_day(
        &self,
        presence: &Presence,
        trips: &[Trip],
    ) -> Result<Presence, DomainError>;

    /// The trip snapshot rows recorded for a presence day, ordered by position.
    async fn list_trips(&self, presence_id: &str) -> Result<Vec<Trip>, DomainError>;

    async fn find_by_id(&self, id: &str) -> Result<Option<Presence>, DomainError>;

    async fn list_by_profile(&self, profile_id: &str) -> Result<Vec<Presence>, DomainError>;

    /// The raw Markdown note of a presence day, or `None` when unset/blank.
    async fn get_note(&self, presence_id: &str) -> Result<Option<String>, DomainError>;

    /// Set (or clear with `None`) the raw Markdown note of a presence day,
    /// stamping `updated_at`.
    async fn set_note(
        &self,
        presence_id: &str,
        note: Option<&str>,
        updated_at: i64,
    ) -> Result<(), DomainError>;

    async fn delete(&self, id: &str) -> Result<(), DomainError>;
}
