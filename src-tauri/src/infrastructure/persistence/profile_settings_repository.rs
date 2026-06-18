use std::sync::Arc;

use async_trait::async_trait;
use libsql::{params, Connection, Row};

use crate::domain::entities::co2_settings::Co2Settings;
use crate::domain::entities::profile_settings::ProfileSettings;
use crate::domain::error::DomainError;
use crate::domain::repositories::profile_settings_repository::ProfileSettingsRepository;
use crate::infrastructure::persistence::db::map_storage;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;

/// Column order shared by the SELECT and [`row_to_settings`].
const COLUMNS: &str = "default_start_minutes, note_font, cell_display_mode, \
    grid_country, default_car_occupancy, include_radiative_forcing, \
    count_building_energy, working_days_per_year, factor_year";

/// Per-profile settings backed by the unlocked vault connection. `load` falls
/// back to defaults when the profile has no row.
pub struct LibsqlProfileSettingsRepository {
    vault: Arc<LibsqlVaultManager>,
}

impl LibsqlProfileSettingsRepository {
    pub fn new(vault: Arc<LibsqlVaultManager>) -> Self {
        Self { vault }
    }

    fn conn(&self) -> Result<Connection, DomainError> {
        self.vault.connection().ok_or(DomainError::Unauthorized)
    }
}

fn row_to_settings(row: &Row) -> Result<ProfileSettings, DomainError> {
    // Booleans are stored as INTEGER 0/1; read as i64 then compare to avoid any
    // libSQL int->bool ambiguity. factor_year is i32 on the entity.
    let include_rf: i64 = row.get(5).map_err(map_storage)?;
    let count_be: i64 = row.get(6).map_err(map_storage)?;
    Ok(ProfileSettings {
        default_start_minutes: row.get(0).map_err(map_storage)?,
        note_font: row.get(1).map_err(map_storage)?,
        cell_display_mode: row.get(2).map_err(map_storage)?,
        co2: Co2Settings {
            grid_country: row.get(3).map_err(map_storage)?,
            default_car_occupancy: row.get(4).map_err(map_storage)?,
            include_radiative_forcing: include_rf != 0,
            count_building_energy: count_be != 0,
            working_days_per_year: row.get(7).map_err(map_storage)?,
            factor_year: row.get::<i64>(8).map_err(map_storage)? as i32,
        },
    })
}

#[async_trait]
impl ProfileSettingsRepository for LibsqlProfileSettingsRepository {
    async fn load(&self, profile_id: &str) -> Result<ProfileSettings, DomainError> {
        let sql = format!("SELECT {COLUMNS} FROM profile_settings WHERE profile_id = ?1");
        let mut rows = self
            .conn()?
            .query(&sql, params![profile_id])
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => row_to_settings(&row),
            None => Ok(ProfileSettings::default()),
        }
    }

    async fn save(&self, profile_id: &str, s: &ProfileSettings) -> Result<(), DomainError> {
        self.conn()?
            .execute(
                "INSERT INTO profile_settings \
                 (profile_id, default_start_minutes, note_font, cell_display_mode, \
                  grid_country, default_car_occupancy, include_radiative_forcing, \
                  count_building_energy, working_days_per_year, factor_year) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
                 ON CONFLICT(profile_id) DO UPDATE SET \
                   default_start_minutes = excluded.default_start_minutes, \
                   note_font = excluded.note_font, \
                   cell_display_mode = excluded.cell_display_mode, \
                   grid_country = excluded.grid_country, \
                   default_car_occupancy = excluded.default_car_occupancy, \
                   include_radiative_forcing = excluded.include_radiative_forcing, \
                   count_building_energy = excluded.count_building_energy, \
                   working_days_per_year = excluded.working_days_per_year, \
                   factor_year = excluded.factor_year",
                params![
                    profile_id,
                    s.default_start_minutes,
                    s.note_font.clone(),
                    s.cell_display_mode.clone(),
                    s.co2.grid_country.clone(),
                    s.co2.default_car_occupancy,
                    s.co2.include_radiative_forcing as i64,
                    s.co2.count_building_energy as i64,
                    s.co2.working_days_per_year,
                    s.co2.factor_year as i64,
                ],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }
}
