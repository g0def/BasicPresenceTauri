/// One task of a work day's hours encoding. Entries are duration-stacked in
/// `position` order (no explicit start times); the day total is the sum of
/// `minutes`. Title/description/color are snapshotted at encode time so later
/// preset edits never rewrite past days.
#[derive(Debug, Clone)]
pub struct WorkEntry {
    pub id: String,
    pub presence_id: String,
    pub title: String,
    pub description: Option<String>,
    /// Duration in minutes (5..=480, multiple of 5).
    pub minutes: i64,
    /// '#RRGGBB' hex color of the donut segment.
    pub color: String,
    /// 0-based chronological order within the day.
    pub position: i64,
}
