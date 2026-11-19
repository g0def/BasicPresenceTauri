//! Attendance export (Odoo `hr.attendance` import): turns a profile's presence
//! days into check-in / check-out pairs. The pure computation lives here; the
//! file format (CSV) is an infrastructure concern behind [`AttendanceExporter`].

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use chrono::{DateTime, NaiveDateTime, TimeDelta};

use crate::domain::entities::presence::{Presence, PresenceType};
use crate::domain::entities::work_day_schedule::WorkDaySchedule;
use crate::domain::error::DomainError;

/// Clock times applied to a day with no usable schedule, in minutes since
/// midnight (`start < end`, validated by `rules::attendance_fallback`) — or to
/// every day when `apply_to_all` is set (the stored schedules are then ignored
/// for the export only; nothing in the vault changes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FallbackHours {
    pub start_minutes: i64,
    pub end_minutes: i64,
    pub apply_to_all: bool,
}

/// One attendance line: a worked day with its (local, naive) clock times.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttendanceRow {
    /// Epoch ms at UTC midnight of the day (the presence key).
    pub day: i64,
    pub check_in: NaiveDateTime,
    pub check_out: NaiveDateTime,
    /// `true` when the fallback hours were used instead of the day's schedule.
    pub used_fallback: bool,
}

/// Only worked days are attendances: vacation and public holidays are not.
pub fn is_attendance_kind(kind: PresenceType) -> bool {
    matches!(kind, PresenceType::Office | PresenceType::Remote)
}

/// A schedule is usable when its end is after its start (a fresh day with no
/// task has `end == start`).
pub fn usable_schedule(schedule: Option<&WorkDaySchedule>) -> Option<(i64, i64)> {
    schedule
        .filter(|s| s.end_minutes > s.start_minutes)
        .map(|s| (s.start_minutes, s.end_minutes))
}

/// Builds the attendance rows of the `selected` days, sorted by day. Days that
/// are not selected, or not office/remote, are skipped. `end_minutes` may
/// exceed 1440 (work past midnight): the check-out then lands on the next day.
pub fn build_attendance_rows(
    presences: &[Presence],
    schedules: &HashMap<String, WorkDaySchedule>,
    selected: &BTreeSet<i64>,
    fallback: FallbackHours,
) -> Result<Vec<AttendanceRow>, DomainError> {
    let mut rows = presences
        .iter()
        .filter(|p| selected.contains(&p.day) && is_attendance_kind(p.kind))
        .map(|p| {
            let actual = if fallback.apply_to_all {
                None
            } else {
                usable_schedule(schedules.get(&p.id))
            };
            let (start, end) = actual.unwrap_or((fallback.start_minutes, fallback.end_minutes));
            let midnight = DateTime::from_timestamp_millis(p.day)
                .ok_or_else(|| DomainError::Validation("day out of range".into()))?
                .naive_utc();
            Ok(AttendanceRow {
                day: p.day,
                check_in: midnight + TimeDelta::minutes(start),
                check_out: midnight + TimeDelta::minutes(end),
                used_fallback: actual.is_none(),
            })
        })
        .collect::<Result<Vec<_>, DomainError>>()?;
    rows.sort_by_key(|r| r.day);
    Ok(rows)
}

/// Port: write attendance rows to a file. Implemented in the infrastructure
/// layer (Odoo-flavoured CSV today).
pub trait AttendanceExporter: Send + Sync {
    fn write_attendances(
        &self,
        employee: &str,
        rows: &[AttendanceRow],
        path: &Path,
    ) -> Result<(), DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rules::MS_PER_DAY;

    /// 2026-10-05, epoch ms at UTC midnight.
    const OCT_5: i64 = 20_731 * MS_PER_DAY;

    fn presence(id: &str, day: i64, kind: PresenceType) -> Presence {
        Presence {
            id: id.to_string(),
            profile_id: "p".to_string(),
            day,
            kind,
            co2_kg: None,
            is_estimated: false,
            created_at: 0,
            updated_at: 0,
            work_minutes: 0,
        }
    }

    fn schedule(id: &str, start: i64, end: i64) -> (String, WorkDaySchedule) {
        (
            id.to_string(),
            WorkDaySchedule {
                presence_id: id.to_string(),
                start_minutes: start,
                end_minutes: end,
            },
        )
    }

    const FALLBACK: FallbackHours = FallbackHours {
        start_minutes: 8 * 60,
        end_minutes: 17 * 60,
        apply_to_all: false,
    };

    fn fmt(dt: NaiveDateTime) -> String {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    #[test]
    fn oct_5_constant_is_the_right_day() {
        let d = DateTime::from_timestamp_millis(OCT_5).unwrap().naive_utc();
        assert_eq!(fmt(d), "2026-10-05 00:00:00");
    }

    #[test]
    fn uses_the_actual_schedule_when_it_has_a_duration() {
        let presences = [presence("a", OCT_5, PresenceType::Office)];
        let schedules = HashMap::from([schedule("a", 8 * 60 + 30, 17 * 60 + 15)]);
        let rows =
            build_attendance_rows(&presences, &schedules, &BTreeSet::from([OCT_5]), FALLBACK)
                .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(fmt(rows[0].check_in), "2026-10-05 08:30:00");
        assert_eq!(fmt(rows[0].check_out), "2026-10-05 17:15:00");
        assert!(!rows[0].used_fallback);
    }

    #[test]
    fn falls_back_without_schedule_or_when_end_equals_start() {
        let day2 = OCT_5 + MS_PER_DAY;
        let presences = [
            presence("a", OCT_5, PresenceType::Remote),
            presence("b", day2, PresenceType::Office),
        ];
        let schedules = HashMap::from([schedule("b", 510, 510)]);
        let rows = build_attendance_rows(
            &presences,
            &schedules,
            &BTreeSet::from([OCT_5, day2]),
            FALLBACK,
        )
        .unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.used_fallback));
        assert_eq!(fmt(rows[0].check_in), "2026-10-05 08:00:00");
        assert_eq!(fmt(rows[1].check_out), "2026-10-06 17:00:00");
    }

    #[test]
    fn apply_to_all_overrides_actual_schedules() {
        let presences = [presence("a", OCT_5, PresenceType::Office)];
        let schedules = HashMap::from([schedule("a", 9 * 60, 18 * 60 + 30)]);
        let rows = build_attendance_rows(
            &presences,
            &schedules,
            &BTreeSet::from([OCT_5]),
            FallbackHours {
                apply_to_all: true,
                ..FALLBACK
            },
        )
        .unwrap();
        assert_eq!(fmt(rows[0].check_in), "2026-10-05 08:00:00");
        assert_eq!(fmt(rows[0].check_out), "2026-10-05 17:00:00");
        assert!(rows[0].used_fallback);
    }

    #[test]
    fn work_past_midnight_checks_out_the_next_day() {
        let presences = [presence("a", OCT_5, PresenceType::Office)];
        let schedules = HashMap::from([schedule("a", 20 * 60, 1440 + 90)]);
        let rows =
            build_attendance_rows(&presences, &schedules, &BTreeSet::from([OCT_5]), FALLBACK)
                .unwrap();
        assert_eq!(fmt(rows[0].check_out), "2026-10-06 01:30:00");
    }

    #[test]
    fn skips_unselected_and_non_worked_days_and_sorts() {
        let presences = [
            presence("c", OCT_5 + 3 * MS_PER_DAY, PresenceType::Office),
            presence("v", OCT_5 + MS_PER_DAY, PresenceType::Vacation),
            presence("h", OCT_5 + 2 * MS_PER_DAY, PresenceType::Holiday),
            presence("a", OCT_5, PresenceType::Remote),
            presence("u", OCT_5 + 4 * MS_PER_DAY, PresenceType::Office),
        ];
        let selected: BTreeSet<i64> = (0..4).map(|i| OCT_5 + i * MS_PER_DAY).collect();
        let rows = build_attendance_rows(&presences, &HashMap::new(), &selected, FALLBACK).unwrap();
        let days: Vec<i64> = rows.iter().map(|r| r.day).collect();
        assert_eq!(days, vec![OCT_5, OCT_5 + 3 * MS_PER_DAY]);
    }
}
