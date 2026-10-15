use std::sync::Arc;

use uuid::Uuid;

use crate::application::dto::work_entry_dto::{WorkDayDto, WorkEntryDto, WorkEntryInputDto};
use crate::domain::entities::presence::PresenceType;
use crate::domain::entities::work_entry::WorkEntry;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::rules;
use crate::domain::services::unit_of_work::UnitOfWork;

/// Atomically replace the work entries of a presence day. Hours only exist on
/// work days, so the target presence must be office/remote. Positions come
/// from array order and entry ids are regenerated (replace-all semantics).
/// When the day has a schedule, its stored end time is recomputed from the
/// new total and returned alongside the entries.
pub struct SetWorkEntriesUseCase {
    entries: Arc<dyn WorkEntryRepository>,
    presences: Arc<dyn PresenceRepository>,
    uow: Arc<dyn UnitOfWork>,
}

impl SetWorkEntriesUseCase {
    pub fn new(
        entries: Arc<dyn WorkEntryRepository>,
        presences: Arc<dyn PresenceRepository>,
        uow: Arc<dyn UnitOfWork>,
    ) -> Self {
        Self {
            entries,
            presences,
            uow,
        }
    }

    pub async fn execute(
        &self,
        presence_id: &str,
        inputs: Vec<WorkEntryInputDto>,
    ) -> Result<WorkDayDto, DomainError> {
        let presence_id = rules::required_id("presenceId", presence_id)?;
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

        rules::work_day_total(&inputs.iter().map(|i| i.minutes).collect::<Vec<_>>())?;
        let mut entries = Vec::with_capacity(inputs.len());
        for (i, input) in inputs.into_iter().enumerate() {
            entries.push(WorkEntry {
                id: Uuid::now_v7().to_string(),
                presence_id: presence_id.to_string(),
                title: rules::title(&input.title)?,
                description: rules::description(input.description.as_deref())?,
                minutes: rules::task_minutes(input.minutes)?,
                color: rules::color(&input.color)?,
                position: i as i64,
            });
        }

        // The entries and the end time derived from them commit together.
        let total: i64 = entries.iter().map(|e| e.minutes).sum();
        let mut schedule = None;
        self.uow
            .run(Box::pin(async {
                self.entries
                    .replace_for_presence(presence_id, &entries)
                    .await?;
                // Keep the stored end time in sync with the new total.
                if let Some(mut s) = self.entries.get_schedule(presence_id).await? {
                    s.end_minutes = s.start_minutes + total;
                    self.entries.set_schedule(&s).await?;
                    schedule = Some(s.into());
                }
                Ok(())
            }))
            .await?;

        Ok(WorkDayDto {
            entries: entries.into_iter().map(WorkEntryDto::from).collect(),
            schedule,
        })
    }
}
