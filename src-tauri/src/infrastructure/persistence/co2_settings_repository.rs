use std::sync::Arc;

use async_trait::async_trait;
use libsql::{params, Connection};

use crate::domain::entities::co2_settings::Co2Settings;
use crate::domain::error::DomainError;
use crate::domain::repositories::co2_settings_repository::Co2SettingsRepository;
use crate::infrastructure::persistence::db::map_storage;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;

const CONFIG_KEY: &str = "co2_config";

/// On-disk JSON shape of the `co2_config` blob. Kept here so serde stays out of
/// the domain entity (transport/persistence concern only).
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Co2ConfigRecord {
    grid_country: String,
    default_car_occupancy: i64,
    include_radiative_forcing: bool,
    count_building_energy: bool,
    working_days_per_year: i64,
    factor_year: i32,
}

impl From<Co2ConfigRecord> for Co2Settings {
    fn from(r: Co2ConfigRecord) -> Self {
        Self {
            grid_country: r.grid_country,
            default_car_occupancy: r.default_car_occupancy,
            include_radiative_forcing: r.include_radiative_forcing,
            count_building_energy: r.count_building_energy,
            working_days_per_year: r.working_days_per_year,
            factor_year: r.factor_year,
        }
    }
}

/// Loads CO2 settings from the `vault_meta` key/value table (key `co2_config`,
/// a JSON blob), falling back to [`Co2Settings::default`] when absent or
/// unparseable. No setter yet — overrides are written directly to the vault.
pub struct LibsqlCo2SettingsRepository {
    vault: Arc<LibsqlVaultManager>,
}

impl LibsqlCo2SettingsRepository {
    pub fn new(vault: Arc<LibsqlVaultManager>) -> Self {
        Self { vault }
    }

    fn conn(&self) -> Result<Connection, DomainError> {
        self.vault.connection().ok_or(DomainError::Unauthorized)
    }
}

#[async_trait]
impl Co2SettingsRepository for LibsqlCo2SettingsRepository {
    async fn load(&self) -> Result<Co2Settings, DomainError> {
        let mut rows = self
            .conn()?
            .query(
                "SELECT value FROM vault_meta WHERE key = ?1",
                params![CONFIG_KEY],
            )
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => {
                let value: String = row.get(0).map_err(map_storage)?;
                Ok(serde_json::from_str::<Co2ConfigRecord>(&value)
                    .map(Co2Settings::from)
                    .unwrap_or_default())
            }
            None => Ok(Co2Settings::default()),
        }
    }
}
