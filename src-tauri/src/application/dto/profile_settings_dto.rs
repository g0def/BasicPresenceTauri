use serde::{Deserialize, Serialize};

use crate::domain::entities::co2_settings::Co2Settings;
use crate::domain::entities::profile_settings::ProfileSettings;

/// Per-profile settings on the wire. CO2 fields are hoisted to the top level
/// (flat, camelCase) — the simplest contract for the settings page and a match
/// for the legacy `co2_config` camelCase keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSettingsDto {
    pub default_start_minutes: i64,
    pub note_font: String,
    pub cell_display_mode: String,
    pub grid_country: String,
    pub default_car_occupancy: i64,
    pub include_radiative_forcing: bool,
    pub count_building_energy: bool,
    pub working_days_per_year: i64,
    pub factor_year: i32,
}

impl From<ProfileSettings> for ProfileSettingsDto {
    fn from(s: ProfileSettings) -> Self {
        Self {
            default_start_minutes: s.default_start_minutes,
            note_font: s.note_font,
            cell_display_mode: s.cell_display_mode,
            grid_country: s.co2.grid_country,
            default_car_occupancy: s.co2.default_car_occupancy,
            include_radiative_forcing: s.co2.include_radiative_forcing,
            count_building_energy: s.co2.count_building_energy,
            working_days_per_year: s.co2.working_days_per_year,
            factor_year: s.co2.factor_year,
        }
    }
}

impl From<ProfileSettingsDto> for ProfileSettings {
    fn from(d: ProfileSettingsDto) -> Self {
        Self {
            default_start_minutes: d.default_start_minutes,
            note_font: d.note_font,
            cell_display_mode: d.cell_display_mode,
            co2: Co2Settings {
                grid_country: d.grid_country,
                default_car_occupancy: d.default_car_occupancy,
                include_radiative_forcing: d.include_radiative_forcing,
                count_building_energy: d.count_building_energy,
                working_days_per_year: d.working_days_per_year,
                factor_year: d.factor_year,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pins the wire contract: these are the exact camelCase keys the frontend
    /// DTO must mirror. A rename here fails loudly instead of silently sending a
    /// field the frontend reads as `undefined` (which surfaced as `NaN:NaN`).
    #[test]
    fn serializes_with_the_expected_camelcase_keys() {
        let dto = ProfileSettingsDto {
            default_start_minutes: 510,
            note_font: "sans".to_string(),
            cell_display_mode: "co2".to_string(),
            grid_country: "BE".to_string(),
            default_car_occupancy: 1,
            include_radiative_forcing: true,
            count_building_energy: false,
            working_days_per_year: 220,
            factor_year: 2025,
        };
        let value = serde_json::to_value(&dto).unwrap();
        let mut keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "cellDisplayMode",
                "countBuildingEnergy",
                "defaultCarOccupancy",
                "defaultStartMinutes",
                "factorYear",
                "gridCountry",
                "includeRadiativeForcing",
                "noteFont",
                "workingDaysPerYear",
            ]
        );
    }
}
