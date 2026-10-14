use std::sync::Arc;

use crate::application::dto::profile_settings_dto::ProfileSettingsDto;
use crate::application::use_cases::set_work_schedule::validate_start;
use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::error::DomainError;
use crate::domain::repositories::emission_factor_repository::EmissionFactorRepository;
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
    factors: Arc<dyn EmissionFactorRepository>,
}

impl SetProfileSettingsUseCase {
    pub fn new(
        settings: Arc<dyn ProfileSettingsRepository>,
        factors: Arc<dyn EmissionFactorRepository>,
    ) -> Self {
        Self { settings, factors }
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
        validate_factor_year(&self.factors, dto.factor_year).await?;

        let settings = ProfileSettings::from(dto);
        self.settings.save(profile_id, &settings).await?;
        Ok(ProfileSettingsDto::from(settings))
    }
}

/// Check the chosen reference year against the vintages actually seeded in the
/// referential.
///
/// Split from [`validate`] because it needs a repository round-trip. It is the
/// one settings field with no static domain: hard-coding the years here would
/// drift the moment a new vintage ships, so the referential is the source of
/// truth. An unknown year is not a cosmetic problem — it makes every subsequent
/// footprint resolve to zero and get snapshotted that way.
pub(crate) async fn validate_factor_year(
    factors: &Arc<dyn EmissionFactorRepository>,
    factor_year: i32,
) -> Result<(), DomainError> {
    let years = factors.available_years().await?;
    if years.is_empty() {
        // Referential not seeded yet (the vault migrations own that); nothing to
        // validate against, and refusing here would block a legitimate setup.
        return Ok(());
    }
    if !years.contains(&factor_year) {
        let known = years
            .iter()
            .map(|y| y.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(DomainError::Validation(format!(
            "invalid factorYear {factor_year} (available: {known})"
        )));
    }
    Ok(())
}

/// Validate every typed field that has a static domain. Mirrors the table
/// CHECKs so a violation surfaces as a clean `Validation` error rather than an
/// opaque `Storage` one. Shared with the bundle import path, which receives
/// untrusted settings from a file.
///
/// `factor_year` is deliberately **not** here — see [`validate_factor_year`],
/// which every caller of this function must also run.
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

    /// Stands in for the seeded referential.
    struct Referential(Vec<i32>);

    #[async_trait::async_trait]
    impl EmissionFactorRepository for Referential {
        async fn list(
            &self,
            _year: i32,
        ) -> Result<Vec<crate::domain::entities::emission_factor::EmissionFactor>, DomainError>
        {
            Ok(Vec::new())
        }
        async fn list_grid_variants(
            &self,
            _year: i32,
        ) -> Result<Vec<crate::domain::entities::emission_factor::GridVariant>, DomainError>
        {
            Ok(Vec::new())
        }
        async fn available_years(&self) -> Result<Vec<i32>, DomainError> {
            Ok(self.0.clone())
        }
    }

    fn referential(years: &[i32]) -> Arc<dyn EmissionFactorRepository> {
        Arc::new(Referential(years.to_vec()))
    }

    #[test]
    fn accepts_canonical_defaults() {
        assert!(validate(&valid()).is_ok());
    }

    /// The gap that let a shared bundle zero out every future footprint: this
    /// field was the only one of the nine with neither a validator nor a CHECK.
    #[test]
    fn rejects_a_vintage_absent_from_the_referential() {
        tauri::async_runtime::block_on(async {
            let factors = referential(&[2025, 2026]);

            assert!(validate_factor_year(&factors, 2026).await.is_ok());
            assert!(validate_factor_year(&factors, 2025).await.is_ok());

            let err = validate_factor_year(&factors, 1999)
                .await
                .expect_err("an unseeded vintage must be refused");
            assert!(matches!(err, DomainError::Validation(_)));
            assert!(validate_factor_year(&factors, 3000).await.is_err());
        });
    }

    /// Before the referential is seeded there is nothing to check against, and
    /// refusing would block a legitimate first run.
    #[test]
    fn tolerates_an_unseeded_referential() {
        tauri::async_runtime::block_on(async {
            assert!(validate_factor_year(&referential(&[]), 2025).await.is_ok());
        });
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
