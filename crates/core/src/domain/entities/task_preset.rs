/// A reusable task preset owned by a profile (e.g. "Réunion équipe, 30min,
/// #4B7F52"). `default_minutes` only seeds the duration when the preset is
/// added to a day; the day's entry keeps its own copy afterwards.
#[derive(Debug, Clone)]
pub struct TaskPreset {
    pub id: String,
    pub profile_id: String,
    pub title: String,
    pub description: Option<String>,
    /// Suggested duration in minutes (5..=480, multiple of 5).
    pub default_minutes: i64,
    /// '#RRGGBB' hex color used for the day-donut segment.
    pub color: String,
    pub created_at: i64,
    pub updated_at: i64,
}
