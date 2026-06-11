use serde::{Deserialize, Serialize};

use crate::domain::entities::work_day_schedule::WorkDaySchedule;
use crate::domain::entities::work_entry::WorkEntry;

/// A day's work entry returned to the frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkEntryDto {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub minutes: i64,
    pub color: String,
    pub position: i64,
}

impl From<WorkEntry> for WorkEntryDto {
    fn from(e: WorkEntry) -> Self {
        Self {
            id: e.id,
            title: e.title,
            description: e.description,
            minutes: e.minutes,
            color: e.color,
            position: e.position,
        }
    }
}

/// A work entry received FROM the frontend on save. The position is the array
/// index and entry ids are regenerated server-side (replace-all semantics).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkEntryInputDto {
    pub title: String,
    pub description: Option<String>,
    pub minutes: i64,
    pub color: String,
}

/// The day's start/end clock times (minutes since midnight). The end is
/// recomputed backend-side on every save: start + sum of entry durations.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkDayScheduleDto {
    pub start_minutes: i64,
    pub end_minutes: i64,
}

impl From<WorkDaySchedule> for WorkDayScheduleDto {
    fn from(s: WorkDaySchedule) -> Self {
        Self {
            start_minutes: s.start_minutes,
            end_minutes: s.end_minutes,
        }
    }
}

/// Result of a day's entries save: the canonical entries AND the schedule,
/// whose end time the save recomputed (when a schedule exists).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkDayDto {
    pub entries: Vec<WorkEntryDto>,
    pub schedule: Option<WorkDayScheduleDto>,
}
