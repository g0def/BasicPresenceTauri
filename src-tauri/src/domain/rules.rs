//! Business invariants on user input, in one place.
//!
//! Every write path — the regular commands *and* the bundle import, which
//! carries untrusted file content — goes through these functions, so a value
//! the rest of the app deems impossible can never reach the vault, and no input
//! is unbounded (memory, CPU of the Markdown renderer, export size).
//!
//! Each function validates *and normalizes* (trims, clamps the documented
//! defaults), returning the value to store.

use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::error::DomainError;

/// Milliseconds in a UTC day.
pub const MS_PER_DAY: i64 = 86_400_000;
/// Minutes in a day.
pub const MINUTES_PER_DAY: i64 = 1_440;

/// Identity fields of a profile (names, company, job title).
pub const MAX_PROFILE_FIELD_LEN: usize = 120;
/// Titles (task presets, work entries) and commute names.
pub const MAX_TITLE_LEN: usize = 200;
/// Free-text descriptions of task presets and work entries.
pub const MAX_DESCRIPTION_LEN: usize = 2_000;
/// A day note, in bytes of Markdown: ~50 pages of text. Bounds the work of the
/// Markdown renderer + syntax highlighter on every read.
pub const MAX_NOTE_BYTES: usize = 100 * 1024;
/// Identifiers sent by the frontend (UUIDs are 36 characters).
pub const MAX_ID_LEN: usize = 64;
/// Segments of a commute, trips of a day.
pub const MAX_SEGMENTS: usize = 20;
/// One-way distance of a single segment, km.
pub const MAX_DISTANCE_KM: f64 = 1_000.0;
/// People sharing a vehicle (the driver included).
pub const MAX_OCCUPANTS: i64 = 9;
/// Work entries of a single day.
pub const MAX_WORK_ENTRIES: usize = 50;
/// Labels supplied for a spreadsheet export.
pub const MAX_EXPORT_LABELS: usize = 200;
/// Emission-factor vintages shipped in the vault referential.
pub const KNOWN_FACTOR_YEARS: &[i32] = &[2025, 2026];

fn invalid(message: impl Into<String>) -> DomainError {
    DomainError::Validation(message.into())
}

/// A required identifier (profile, presence, commute… id).
pub fn required_id<'a>(field: &str, value: &'a str) -> Result<&'a str, DomainError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(invalid(format!("{field} is required")));
    }
    if value.len() > MAX_ID_LEN {
        return Err(invalid(format!("{field} is too long")));
    }
    Ok(value)
}

/// A required, trimmed text field of bounded length.
pub fn required_text(field: &str, value: &str, max_len: usize) -> Result<String, DomainError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(invalid(format!("{field} is required")));
    }
    if value.chars().count() > max_len {
        return Err(invalid(format!(
            "{field} must be at most {max_len} characters"
        )));
    }
    Ok(value.to_string())
}

/// An optional trimmed text field: blank becomes `None`; length is bounded.
pub fn optional_text(
    field: &str,
    value: Option<&str>,
    max_len: usize,
) -> Result<Option<String>, DomainError> {
    match value.map(str::trim).filter(|v| !v.is_empty()) {
        Some(v) if v.chars().count() > max_len => Err(invalid(format!(
            "{field} must be at most {max_len} characters"
        ))),
        Some(v) => Ok(Some(v.to_string())),
        None => Ok(None),
    }
}

/// A profile identity field (first/last name, company).
pub fn profile_field(field: &str, value: &str) -> Result<String, DomainError> {
    required_text(field, value, MAX_PROFILE_FIELD_LEN)
}

/// The optional job title of a profile.
pub fn profile_optional_field(
    field: &str,
    value: Option<&str>,
) -> Result<Option<String>, DomainError> {
    optional_text(field, value, MAX_PROFILE_FIELD_LEN)
}

/// A presence `day`: a non-negative multiple of [`MS_PER_DAY`] (epoch ms at UTC
/// midnight), matching the key the frontend sends. The vault has no CHECK on
/// the column, so every write path must enforce this.
pub fn day(day: i64) -> Result<i64, DomainError> {
    if day < 0 || day % MS_PER_DAY != 0 {
        return Err(invalid("day must be epoch ms at UTC midnight"));
    }
    Ok(day)
}

/// A start time inside one day (minutes since midnight).
pub fn start_minutes(start: i64) -> Result<i64, DomainError> {
    if !(0..MINUTES_PER_DAY).contains(&start) {
        return Err(invalid("startMinutes must be between 0 and 1439"));
    }
    Ok(start)
}

/// Title of a task preset or work entry.
pub fn title(title: &str) -> Result<String, DomainError> {
    required_text("title", title, MAX_TITLE_LEN)
}

/// Optional description of a task preset or work entry.
pub fn description(description: Option<&str>) -> Result<Option<String>, DomainError> {
    optional_text("description", description, MAX_DESCRIPTION_LEN)
}

/// Duration of a task: 5 to 480 minutes, on the slider's 5-minute grid.
pub fn task_minutes(minutes: i64) -> Result<i64, DomainError> {
    if !(5..=480).contains(&minutes) || minutes % 5 != 0 {
        return Err(invalid("minutes must be between 5 and 480 in steps of 5"));
    }
    Ok(minutes)
}

/// A `#RRGGBB` color.
pub fn color(color: &str) -> Result<String, DomainError> {
    let color = color.trim();
    let hex = color.strip_prefix('#').unwrap_or("");
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(invalid("color must be a #RRGGBB hex value"));
    }
    Ok(color.to_string())
}

/// The work entries of one day: bounded count, and at most one day of work.
pub fn work_day_total(minutes: &[i64]) -> Result<i64, DomainError> {
    if minutes.len() > MAX_WORK_ENTRIES {
        return Err(invalid(format!(
            "a day holds at most {MAX_WORK_ENTRIES} entries"
        )));
    }
    let total: i64 = minutes.iter().sum();
    if total > MINUTES_PER_DAY {
        return Err(invalid("a day's entries cannot exceed 24 hours"));
    }
    Ok(total)
}

/// A day note: blank clears it; otherwise trimmed and size-bounded.
pub fn note(markdown: Option<&str>) -> Result<Option<String>, DomainError> {
    let trimmed = markdown.map(str::trim).unwrap_or("");
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() > MAX_NOTE_BYTES {
        return Err(invalid(format!(
            "a note is limited to {} KiB",
            MAX_NOTE_BYTES / 1024
        )));
    }
    Ok(Some(trimmed.to_string()))
}

/// A transport-mode id (referential key).
pub fn mode_id(position: usize, mode_id: &str) -> Result<String, DomainError> {
    let mode_id = mode_id.trim();
    if mode_id.is_empty() {
        return Err(invalid(format!("segment {position}: mode is required")));
    }
    if mode_id.len() > MAX_ID_LEN {
        return Err(invalid(format!("segment {position}: invalid mode")));
    }
    Ok(mode_id.to_string())
}

/// A one-way distance in km: finite, `0..=MAX_DISTANCE_KM`.
pub fn distance_km(position: usize, km: f64) -> Result<f64, DomainError> {
    if !km.is_finite() || !(0.0..=MAX_DISTANCE_KM).contains(&km) {
        return Err(invalid(format!(
            "segment {position}: distance must be between 0 and {MAX_DISTANCE_KM} km"
        )));
    }
    Ok(km)
}

/// Vehicle occupants: missing means 1 (alone), and at most [`MAX_OCCUPANTS`].
pub fn occupants(position: usize, occupants: Option<i64>) -> Result<i64, DomainError> {
    let n = occupants.unwrap_or(1).max(1);
    if n > MAX_OCCUPANTS {
        return Err(invalid(format!(
            "segment {position}: at most {MAX_OCCUPANTS} occupants"
        )));
    }
    Ok(n)
}

/// Number of segments of a commute / trips of a day.
pub fn segment_count(count: usize, required: bool) -> Result<(), DomainError> {
    if required && count == 0 {
        return Err(invalid("at least one segment is required"));
    }
    if count > MAX_SEGMENTS {
        return Err(invalid(format!("at most {MAX_SEGMENTS} segments")));
    }
    Ok(())
}

/// A CO2 amount (kg): absent, or finite and non-negative. A hand-edited bundle
/// could carry NaN/Inf/negative and SQLite would store it, poisoning later
/// aggregation.
pub fn co2_kg(co2_kg: Option<f64>) -> Result<Option<f64>, DomainError> {
    if let Some(c) = co2_kg
        && (!c.is_finite() || c < 0.0)
    {
        return Err(invalid("co2 must be a finite value >= 0"));
    }
    Ok(co2_kg)
}

/// An emission-factor vintage present in the referential.
pub fn factor_year(year: i64) -> Result<i32, DomainError> {
    i32::try_from(year)
        .ok()
        .filter(|y| KNOWN_FACTOR_YEARS.contains(y))
        .ok_or_else(|| invalid("unknown emission-factor year"))
}

/// Allowed enum values — kept in sync with the frontend literals and the
/// `0010` table CHECK constraints (defence in depth).
const NOTE_FONTS: &[&str] = &["sans", "serif", "mono"];
const CELL_MODES: &[&str] = &["co2", "hours"];
const GRID_COUNTRIES: &[&str] = &["FR", "BE", "DE", "EU"];

/// Every typed field of a profile's settings. Mirrors the table CHECKs (so a
/// violation surfaces as a clean `Validation` error rather than an opaque
/// `Storage` one) and adds the bounds the schema lacks.
pub fn profile_settings(s: &ProfileSettings) -> Result<(), DomainError> {
    start_minutes(s.default_start_minutes)?;
    if !NOTE_FONTS.contains(&s.note_font.as_str()) {
        return Err(invalid("invalid noteFont"));
    }
    if !CELL_MODES.contains(&s.cell_display_mode.as_str()) {
        return Err(invalid("invalid cellDisplayMode"));
    }
    if !GRID_COUNTRIES.contains(&s.co2.grid_country.as_str()) {
        return Err(invalid("invalid gridCountry"));
    }
    if !(1..=MAX_OCCUPANTS).contains(&s.co2.default_car_occupancy) {
        return Err(invalid(format!(
            "defaultCarOccupancy must be between 1 and {MAX_OCCUPANTS}"
        )));
    }
    if !(0..=366).contains(&s.co2.working_days_per_year) {
        return Err(invalid("workingDaysPerYear must be between 0 and 366"));
    }
    factor_year(i64::from(s.co2.factor_year))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_is_epoch_ms_at_utc_midnight() {
        assert!(day(0).is_ok());
        assert!(day(MS_PER_DAY * 20_000).is_ok());
        assert!(day(-MS_PER_DAY).is_err());
        assert!(day(MS_PER_DAY + 1).is_err());
    }

    #[test]
    fn start_must_be_a_clock_time_inside_one_day() {
        assert!(start_minutes(0).is_ok());
        assert!(start_minutes(510).is_ok()); // 8h30
        assert!(start_minutes(1439).is_ok());
        assert!(start_minutes(-1).is_err());
        assert!(start_minutes(1440).is_err());
    }

    #[test]
    fn minutes_must_stay_on_the_5_min_grid_within_bounds() {
        assert!(task_minutes(5).is_ok());
        assert!(task_minutes(480).is_ok());
        assert!(task_minutes(125).is_ok());
        assert!(task_minutes(0).is_err());
        assert!(task_minutes(485).is_err());
        assert!(task_minutes(7).is_err());
        assert!(task_minutes(-5).is_err());
    }

    #[test]
    fn color_must_be_rrggbb_hex() {
        assert!(color("#4B7F52").is_ok());
        assert!(color(" #4b7f52 ").is_ok());
        assert!(color("4B7F52").is_err());
        assert!(color("#4B7F5").is_err());
        assert!(color("#4B7F5G").is_err());
        assert!(color("").is_err());
    }

    #[test]
    fn titles_are_trimmed_required_and_bounded() {
        assert_eq!(title("  Réunion  ").unwrap(), "Réunion");
        assert!(title("   ").is_err());
        assert!(title(&"é".repeat(MAX_TITLE_LEN)).is_ok());
        assert!(title(&"é".repeat(MAX_TITLE_LEN + 1)).is_err());
    }

    #[test]
    fn blank_descriptions_collapse_to_none() {
        assert_eq!(description(Some("  ")).unwrap(), None);
        assert_eq!(description(Some(" notes ")).unwrap(), Some("notes".into()));
        assert_eq!(description(None).unwrap(), None);
        assert!(description(Some(&"x".repeat(MAX_DESCRIPTION_LEN + 1))).is_err());
    }

    #[test]
    fn notes_are_bounded_and_blank_clears() {
        assert_eq!(note(Some("  \n ")).unwrap(), None);
        assert_eq!(note(None).unwrap(), None);
        assert_eq!(note(Some(" # Hi ")).unwrap(), Some("# Hi".into()));
        assert!(note(Some(&"x".repeat(MAX_NOTE_BYTES + 1))).is_err());
    }

    #[test]
    fn distances_and_occupants_are_bounded() {
        assert!(distance_km(0, 0.0).is_ok());
        assert!(distance_km(0, MAX_DISTANCE_KM).is_ok());
        assert!(distance_km(0, -0.1).is_err());
        assert!(distance_km(0, f64::NAN).is_err());
        assert!(distance_km(0, f64::INFINITY).is_err());
        assert!(distance_km(0, MAX_DISTANCE_KM + 1.0).is_err());
        assert_eq!(occupants(0, None).unwrap(), 1);
        assert_eq!(occupants(0, Some(0)).unwrap(), 1);
        assert!(occupants(0, Some(MAX_OCCUPANTS + 1)).is_err());
    }

    #[test]
    fn a_day_holds_at_most_24_hours_of_work() {
        assert_eq!(work_day_total(&[480, 480, 480]).unwrap(), 1440);
        assert!(work_day_total(&[480, 480, 480, 5]).is_err());
        assert!(work_day_total(&vec![5; MAX_WORK_ENTRIES + 1]).is_err());
    }

    #[test]
    fn co2_accepts_none_and_finite_nonneg_only() {
        assert!(co2_kg(None).is_ok());
        assert!(co2_kg(Some(0.0)).is_ok());
        assert!(co2_kg(Some(12.5)).is_ok());
        assert!(co2_kg(Some(-0.1)).is_err());
        assert!(co2_kg(Some(f64::NAN)).is_err());
        assert!(co2_kg(Some(f64::INFINITY)).is_err());
    }

    #[test]
    fn only_known_factor_years_are_accepted() {
        assert_eq!(factor_year(2026).unwrap(), 2026);
        assert!(factor_year(2024).is_err());
        assert!(factor_year(i64::MAX).is_err());
    }

    fn settings() -> ProfileSettings {
        ProfileSettings::default()
    }

    #[test]
    fn settings_defaults_are_valid() {
        assert!(profile_settings(&settings()).is_ok());
    }

    #[test]
    fn settings_reject_unknown_enums() {
        let mut s = settings();
        s.note_font = "comic".to_string();
        assert!(profile_settings(&s).is_err());

        let mut s = settings();
        s.cell_display_mode = "pie".to_string();
        assert!(profile_settings(&s).is_err());

        let mut s = settings();
        s.co2.grid_country = "US".to_string();
        assert!(profile_settings(&s).is_err());
    }

    #[test]
    fn settings_reject_out_of_range_numbers() {
        let mut s = settings();
        s.default_start_minutes = 1440;
        assert!(profile_settings(&s).is_err());

        let mut s = settings();
        s.co2.default_car_occupancy = 0;
        assert!(profile_settings(&s).is_err());

        let mut s = settings();
        s.co2.working_days_per_year = -1;
        assert!(profile_settings(&s).is_err());

        let mut s = settings();
        s.co2.factor_year = 1999;
        assert!(profile_settings(&s).is_err());
    }
}
