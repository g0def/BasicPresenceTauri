use std::sync::Arc;

use crate::application::dto::presence_dto::PresenceDto;
use crate::application::dto::trip_dto::TripInputDto;
use crate::application::dto::work_entry_dto::WorkEntryInputDto;
use crate::application::use_cases::set_presence::SetPresenceUseCase;
use crate::application::use_cases::set_work_entries::SetWorkEntriesUseCase;
use crate::application::use_cases::set_work_schedule::SetWorkScheduleUseCase;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::rules;
use crate::domain::services::unit_of_work::UnitOfWork;

/// Replay a fully-encoded day onto another day of the same profile: presence
/// type, commute trips, start time and tasks. The target ends up an exact copy
/// of the source (its previous encoding is overwritten), and everything commits
/// as ONE transaction — a failure leaves the target untouched.
///
/// Replaces the frontend "day clipboard" that chained `set_presence`,
/// `set_work_schedule` and `set_work_entries` as three separate IPC calls.
/// Leftover work hours of a former work day are wiped by the vault's
/// type-change triggers when a non-work day is pasted over it.
pub struct CopyDayUseCase {
    presences: Arc<dyn PresenceRepository>,
    entries: Arc<dyn WorkEntryRepository>,
    set_presence: SetPresenceUseCase,
    set_schedule: SetWorkScheduleUseCase,
    set_entries: SetWorkEntriesUseCase,
    uow: Arc<dyn UnitOfWork>,
}

impl CopyDayUseCase {
    pub fn new(
        presences: Arc<dyn PresenceRepository>,
        entries: Arc<dyn WorkEntryRepository>,
        set_presence: SetPresenceUseCase,
        set_schedule: SetWorkScheduleUseCase,
        set_entries: SetWorkEntriesUseCase,
        uow: Arc<dyn UnitOfWork>,
    ) -> Self {
        Self {
            presences,
            entries,
            set_presence,
            set_schedule,
            set_entries,
            uow,
        }
    }

    /// Copy the day `source_presence_id` onto `target_day` (epoch-ms, UTC
    /// midnight) of the same profile, and return the target's new presence.
    pub async fn execute(
        &self,
        source_presence_id: &str,
        target_day: i64,
    ) -> Result<PresenceDto, DomainError> {
        let source_id = rules::required_id("presenceId", source_presence_id)?;
        let source = self
            .presences
            .find_by_id(source_id)
            .await?
            .ok_or(DomainError::NotFound)?;

        let trips: Vec<TripInputDto> = self
            .presences
            .list_trips(source_id)
            .await?
            .into_iter()
            .map(|t| TripInputDto {
                mode_id: t.mode_id,
                distance_km: t.distance_km,
                round_trip: t.round_trip,
                occupants: Some(t.occupants),
            })
            .collect();

        let is_work = source.kind.is_work();
        let (start_minutes, tasks) = if is_work {
            let start = self
                .entries
                .get_schedule(source_id)
                .await?
                .map(|s| s.start_minutes);
            let tasks: Vec<WorkEntryInputDto> = self
                .entries
                .list_by_presence(source_id)
                .await?
                .into_iter()
                .map(|e| WorkEntryInputDto {
                    title: e.title,
                    description: e.description,
                    minutes: e.minutes,
                    color: e.color,
                })
                .collect();
            (start, tasks)
        } else {
            (None, Vec::new())
        };

        let mut saved = None;
        self.uow
            .run(Box::pin(async {
                let presence = self
                    .set_presence
                    .execute(&source.profile_id, target_day, source.kind.as_str(), trips)
                    .await?;
                if is_work {
                    // Start first, then the entries: set_work_entries derives
                    // the end time from start + sum(minutes).
                    if let Some(start) = start_minutes {
                        self.set_schedule.execute(&presence.id, start).await?;
                    }
                    // Replace-all: also clears the target's former tasks.
                    self.set_entries.execute(&presence.id, tasks).await?;
                }
                saved = Some(presence.id);
                Ok(())
            }))
            .await?;

        let id = saved.ok_or_else(|| DomainError::Storage("day not copied".into()))?;
        // Re-read: work minutes were recomputed by the entries write.
        let presence = self
            .presences
            .find_by_id(&id)
            .await?
            .ok_or(DomainError::NotFound)?;
        Ok(PresenceDto::from(presence))
    }
}
