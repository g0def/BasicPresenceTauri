use crate::domain::entities::co2_settings::Co2Settings;

/// Per-profile settings. One conceptual row per profile; the repository returns
/// these defaults when a profile has no stored row (lazy default, mirroring
/// [`Co2SettingsRepository::load`](crate::domain::repositories::co2_settings_repository)).
/// The CO2 config is embedded by value: [`Co2Settings`] stays the unit consumed
/// by `Co2Calculator` / `set_presence`.
#[derive(Debug, Clone)]
pub struct ProfileSettings {
    /// Default work-day start, seeded into `work_day_schedule` at presence
    /// creation for office/remote days. Minutes since midnight, `0..=1439`.
    pub default_start_minutes: i64,
    pub note_font: String,
    pub cell_display_mode: String,
    pub co2: Co2Settings,
}

impl Default for ProfileSettings {
    fn default() -> Self {
        Self {
            // MUST match the `0010` column defaults and the frontend fallback.
            default_start_minutes: 510, // 08:30
            note_font: "sans".to_string(),
            cell_display_mode: "co2".to_string(),
            co2: Co2Settings::default(),
        }
    }
}
