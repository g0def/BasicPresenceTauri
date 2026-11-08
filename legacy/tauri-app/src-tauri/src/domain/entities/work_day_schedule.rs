/// Start/end clock times of an encoded work day, in minutes since midnight.
/// Optional (one row per presence at most). The end is DERIVED — the backend
/// recomputes start + sum of the entries' durations on every save — but
/// stored, so the vault row is the single displayed source of truth. It may
/// exceed 1440 when the tasks run past midnight.
#[derive(Debug, Clone)]
pub struct WorkDaySchedule {
    pub presence_id: String,
    pub start_minutes: i64,
    pub end_minutes: i64,
}
