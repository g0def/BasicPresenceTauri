use std::sync::Arc;

use crate::application::dto::work_entry_dto::WorkDayScheduleDto;
use crate::domain::entities::presence::PresenceType;
use crate::domain::entities::work_day_schedule::WorkDaySchedule;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;

/// Set (upsert) the start time of an office/remote presence day. The stored
/// end time is recomputed here: start + sum of the day's entry durations.
pub struct SetWorkScheduleUseCase {
    entries: Arc<dyn WorkEntryRepository>,
    presences: Arc<dyn PresenceRepository>,
}

impl SetWorkScheduleUseCase {
    pub fn new(
        entries: Arc<dyn WorkEntryRepository>,
        presences: Arc<dyn PresenceRepository>,
    ) -> Self {
        Self { entries, presences }
    }

    pub async fn execute(
        &self,
        presence_id: &str,
        start_minutes: i64,
    ) -> Result<WorkDayScheduleDto, DomainError> {
        let presence_id = presence_id.trim();
        if presence_id.is_empty() {
            return Err(DomainError::Validation(
                "presenceId is required".to_string(),
            ));
        }
        validate_start(start_minutes)?;

        let presence = self
            .presences
            .find_by_id(presence_id)
            .await?
            .ok_or_else(|| DomainError::Validation("presence not found".to_string()))?;
        if !matches!(presence.kind, PresenceType::Office | PresenceType::Remote) {
            return Err(DomainError::Validation(
                "work hours can only be encoded on office/remote days".to_string(),
            ));
        }

        let total: i64 = self
            .entries
            .list_by_presence(presence_id)
            .await?
            .iter()
            .map(|e| e.minutes)
            .sum();
        let schedule = WorkDaySchedule {
            presence_id: presence_id.to_string(),
            start_minutes,
            end_minutes: start_minutes + total,
        };
        self.entries.set_schedule(&schedule).await?;
        Ok(WorkDayScheduleDto::from(schedule))
    }
}

/// The start must be a clock time inside one day.
pub fn validate_start(start_minutes: i64) -> Result<(), DomainError> {
    if !(0..1440).contains(&start_minutes) {
        return Err(DomainError::Validation(
            "startMinutes must be between 0 and 1439".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_must_be_a_clock_time_inside_one_day() {
        assert!(validate_start(0).is_ok());
        assert!(validate_start(510).is_ok()); // 8h30
        assert!(validate_start(1439).is_ok());
        assert!(validate_start(-1).is_err());
        assert!(validate_start(1440).is_err());
    }
}
