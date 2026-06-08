use async_trait::async_trait;

use crate::domain::entities::presence::Presence;
use crate::domain::entities::trip::Trip;
use crate::domain::error::DomainError;

/// Insert/update tallies from a bulk import (domain-level, DTO-agnostic). The
/// number of *skipped* rows is derived by the caller as `total - inserted -
/// updated`, since only the inserted/updated split needs the DB to compute it.
#[derive(Debug, Default, Clone, Copy)]
pub struct ImportCounts {
    pub inserted: u32,
    pub updated: u32,
}

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

    async fn list_by_profile(&self, profile_id: &str) -> Result<Vec<Presence>, DomainError>;

    async fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// Bulk-insert presences for a profile in a single transaction.
    ///
    /// `replace_existing == false` keeps existing `(profile_id, day)` rows
    /// (`INSERT … ON CONFLICT DO NOTHING`); `true` overwrites them
    /// (`INSERT … ON CONFLICT DO UPDATE`), preserving the original
    /// `id`/`created_at`. Returns the inserted-vs-updated split.
    async fn import_many(
        &self,
        profile_id: &str,
        entries: &[Presence],
        replace_existing: bool,
    ) -> Result<ImportCounts, DomainError>;
}
