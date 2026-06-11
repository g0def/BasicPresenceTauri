use async_trait::async_trait;

use crate::domain::entities::work_day_schedule::WorkDaySchedule;
use crate::domain::entities::work_entry::WorkEntry;
use crate::domain::error::DomainError;

/// Persistence contract for a day's work hours (entries + optional start/end
/// schedule), implemented over the unlocked vault connection. The day's entry
/// set is replaced as a whole: entry ids are not surfaced as stable handles.
#[async_trait]
pub trait WorkEntryRepository: Send + Sync {
    /// The entries recorded for a presence day, ordered by position.
    async fn list_by_presence(&self, presence_id: &str) -> Result<Vec<WorkEntry>, DomainError>;

    /// Atomically replace the full entry set of a presence day. Passing an
    /// empty slice clears the day.
    async fn replace_for_presence(
        &self,
        presence_id: &str,
        entries: &[WorkEntry],
    ) -> Result<(), DomainError>;

    /// The day's start/end times, if the user set them.
    async fn get_schedule(&self, presence_id: &str)
        -> Result<Option<WorkDaySchedule>, DomainError>;

    /// Create or update the day's start/end times.
    async fn set_schedule(&self, schedule: &WorkDaySchedule) -> Result<(), DomainError>;
}
