use std::sync::Arc;

use crate::application::dto::profile_settings_dto::ProfileSettingsDto;
use crate::application::use_cases::set_work_schedule::validate_start;
use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::error::DomainError;
use crate::domain::repositories::profile_settings_repository::ProfileSettingsRepository;

/// Allowed enum values — kept in sync with the frontend literals and the
/// `0010` table CHECK constraints (defence in depth).
const NOTE_FONTS: &[&str] = &["sans", "serif", "mono"];
const CELL_MODES: &[&str] = &["co2", "hours"];
const GRID_COUNTRIES: &[&str] = &["FR", "BE", "DE", "EU"];

/// Persist a profile's full settings row (the frontend merges partial edits onto
/// the current row before sending the canonical whole).
pub struct SetProfileSettingsUseCase {
    settings: Arc<dyn ProfileSettingsRepository>,
}

impl SetProfileSettingsUseCase {
    pub fn new(settings: Arc<dyn ProfileSettingsRepository>) -> Self {
        Self { settings }
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        dto: ProfileSettingsDto,
    ) -> Result<ProfileSettingsDto, DomainError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(DomainError::Validation("profileId is required".to_string()));
        }
        validate(&dto)?;

        let settings = ProfileSettings::from(dto);
        self.settings.save(profile_id, &settings).await?;
        Ok(ProfileSettingsDto::from(settings))
    }
}

/// Validate every typed field. Mirrors the table CHECKs so a violation surfaces
/// as a clean `Validation` error rather than an opaque `Storage` one. Shared with
/// the bundle import path, which receives untrusted settings from a file.
pub(crate) fn validate(dto: &ProfileSettingsDto) -> Result<(), DomainError> {
    validate_start(dto.default_start_minutes)?;
    if !NOTE_FONTS.contains(&dto.note_font.as_str()) {
        return Err(DomainError::Validation("invalid noteFont".to_string()));
    }
    if !CELL_MODES.contains(&dto.cell_display_mode.as_str()) {
        return Err(DomainError::Validation(
            "invalid cellDisplayMode".to_string(),
        ));
    }
    if !GRID_COUNTRIES.contains(&dto.grid_country.as_str()) {
        return Err(DomainError::Validation("invalid gridCountry".to_string()));
    }
    if dto.default_car_occupancy < 1 {
        return Err(DomainError::Validation(
            "defaultCarOccupancy must be >= 1".to_string(),
        ));
    }
    if dto.working_days_per_year < 0 {
        return Err(DomainError::Validation(
            "workingDaysPerYear must be >= 0".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> ProfileSettingsDto {
        ProfileSettingsDto {
            default_start_minutes: 510,
            note_font: "sans".to_string(),
            cell_display_mode: "co2".to_string(),
            grid_country: "BE".to_string(),
            default_car_occupancy: 1,
            include_radiative_forcing: true,
            count_building_energy: false,
            working_days_per_year: 220,
            factor_year: 2025,
        }
    }

    #[test]
    fn accepts_canonical_defaults() {
        assert!(validate(&valid()).is_ok());
    }

    #[test]
    fn rejects_unknown_enums() {
        let mut d = valid();
        d.note_font = "comic".to_string();
        assert!(validate(&d).is_err());

        let mut d = valid();
        d.cell_display_mode = "pie".to_string();
        assert!(validate(&d).is_err());

        let mut d = valid();
        d.grid_country = "US".to_string();
        assert!(validate(&d).is_err());
    }

    #[test]
    fn rejects_out_of_range_start() {
        let mut d = valid();
        d.default_start_minutes = -1;
        assert!(validate(&d).is_err());
        d.default_start_minutes = 1440;
        assert!(validate(&d).is_err());
    }

    #[test]
    fn rejects_zero_occupancy_and_negative_working_days() {
        let mut d = valid();
        d.default_car_occupancy = 0;
        assert!(validate(&d).is_err());

        let mut d = valid();
        d.working_days_per_year = -1;
        assert!(validate(&d).is_err());
    }
}
