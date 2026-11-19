use std::sync::Arc;

use crate::application::dto::attendance_export_dto::AttendanceDayDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::rules;
use crate::domain::services::attendance_export::is_attendance_kind;

/// List a profile's worked (office/remote) days with their stored schedule,
/// sorted by day — the candidates of an attendance export.
pub struct ListAttendanceDaysUseCase {
    presences: Arc<dyn PresenceRepository>,
    work_entries: Arc<dyn WorkEntryRepository>,
}

impl ListAttendanceDaysUseCase {
    pub fn new(
        presences: Arc<dyn PresenceRepository>,
        work_entries: Arc<dyn WorkEntryRepository>,
    ) -> Self {
        Self {
            presences,
            work_entries,
        }
    }

    pub async fn execute(&self, profile_id: &str) -> Result<Vec<AttendanceDayDto>, DomainError> {
        let profile_id = rules::required_id("profileId", profile_id)?;
        let presences = self.presences.list_by_profile(profile_id).await?;
        let schedules = self
            .work_entries
            .list_schedules_by_profile(profile_id)
            .await?;
        let mut days: Vec<AttendanceDayDto> = presences
            .into_iter()
            .filter(|p| is_attendance_kind(p.kind))
            .map(|p| {
                let schedule = schedules.get(&p.id);
                AttendanceDayDto {
                    day: p.day,
                    kind: p.kind.as_str().to_string(),
                    start_minutes: schedule.map(|s| s.start_minutes),
                    end_minutes: schedule.map(|s| s.end_minutes),
                }
            })
            .collect();
        days.sort_by_key(|d| d.day);
        Ok(days)
    }
}
