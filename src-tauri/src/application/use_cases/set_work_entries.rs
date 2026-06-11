use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::work_entry_dto::{WorkDayDto, WorkEntryDto, WorkEntryInputDto};
use crate::application::use_cases::create_task_preset::{
    normalize_description, validate_color, validate_minutes, validate_title,
};
use crate::domain::entities::presence::PresenceType;
use crate::domain::entities::work_entry::WorkEntry;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;

/// Atomically replace the work entries of a presence day. Hours only exist on
/// work days, so the target presence must be office/remote. Positions come
/// from array order and entry ids are regenerated (replace-all semantics).
/// When the day has a schedule, its stored end time is recomputed from the
/// new total and returned alongside the entries.
pub struct SetWorkEntriesUseCase {
    entries: Arc<dyn WorkEntryRepository>,
    presences: Arc<dyn PresenceRepository>,
}

impl SetWorkEntriesUseCase {
    pub fn new(
        entries: Arc<dyn WorkEntryRepository>,
        presences: Arc<dyn PresenceRepository>,
    ) -> Self {
        Self { entries, presences }
    }

    pub async fn execute(
        &self,
        presence_id: &str,
        inputs: Vec<WorkEntryInputDto>,
    ) -> Result<WorkDayDto, DomainError> {
        let presence_id = presence_id.trim();
        if presence_id.is_empty() {
            return Err(DomainError::Validation(
                "presenceId is required".to_string(),
            ));
        }
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

        let mut entries = Vec::with_capacity(inputs.len());
        for (i, input) in inputs.into_iter().enumerate() {
            entries.push(WorkEntry {
                id: Uuid::now_v7().to_string(),
                presence_id: presence_id.to_string(),
                title: validate_title(&input.title)?,
                description: normalize_description(input.description),
                minutes: validate_minutes(input.minutes)?,
                color: validate_color(&input.color)?,
                position: i as i64,
            });
        }

        self.entries
            .replace_for_presence(presence_id, &entries)
            .await?;

        // Keep the stored end time in sync with the new total.
        let total: i64 = entries.iter().map(|e| e.minutes).sum();
        let schedule = match self.entries.get_schedule(presence_id).await? {
            Some(mut schedule) => {
                schedule.end_minutes = schedule.start_minutes + total;
                self.entries.set_schedule(&schedule).await?;
                Some(schedule.into())
            }
            None => None,
        };

        Ok(WorkDayDto {
            entries: entries.into_iter().map(WorkEntryDto::from).collect(),
            schedule,
        })
    }
}
