use std::sync::Arc;

use async_trait::async_trait;
use libsql::{params, Connection, Row};

use crate::domain::entities::emission_factor::{
    EmissionCategory, EmissionFactor, EmissionUnit, GridVariant,
};
use crate::domain::error::DomainError;
use crate::domain::repositories::emission_factor_repository::EmissionFactorRepository;
use crate::infrastructure::persistence::db::map_storage;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;

const SELECT_COLUMNS: &str = "id, label, value, unit, category, is_param";

/// Read-only emission-factor referential backed by the unlocked vault connection.
pub struct LibsqlEmissionFactorRepository {
    vault: Arc<LibsqlVaultManager>,
}

impl LibsqlEmissionFactorRepository {
    pub fn new(vault: Arc<LibsqlVaultManager>) -> Self {
        Self { vault }
    }

    fn conn(&self) -> Result<Connection, DomainError> {
        self.vault.connection().ok_or(DomainError::Unauthorized)
    }
}

fn row_to_factor(row: &Row) -> Result<EmissionFactor, DomainError> {
    let unit: String = row.get(3).map_err(map_storage)?;
    let category: String = row.get(4).map_err(map_storage)?;
    let is_param: i64 = row.get(5).map_err(map_storage)?;
    Ok(EmissionFactor {
        id: row.get(0).map_err(map_storage)?,
        label: row.get(1).map_err(map_storage)?,
        value: row.get(2).map_err(map_storage)?,
        unit: EmissionUnit::parse(&unit)?,
        category: EmissionCategory::parse(&category)?,
        is_param: is_param != 0,
    })
}

fn row_to_variant(row: &Row) -> Result<GridVariant, DomainError> {
    Ok(GridVariant {
        mode_id: row.get(0).map_err(map_storage)?,
        country: row.get(1).map_err(map_storage)?,
        value: row.get(2).map_err(map_storage)?,
    })
}

#[async_trait]
impl EmissionFactorRepository for LibsqlEmissionFactorRepository {
    async fn list(&self, year: i32) -> Result<Vec<EmissionFactor>, DomainError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM emission_factor WHERE year = ?1 ORDER BY category, id"
        );
        let mut rows = self
            .conn()?
            .query(&sql, params![i64::from(year)])
            .await
            .map_err(map_storage)?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().await.map_err(map_storage)? {
            out.push(row_to_factor(&row)?);
        }
        Ok(out)
    }

    async fn list_grid_variants(&self, year: i32) -> Result<Vec<GridVariant>, DomainError> {
        let mut rows = self
            .conn()?
            .query(
                "SELECT mode_id, country, value FROM emission_factor_grid_variant \
                 WHERE year = ?1",
                params![i64::from(year)],
            )
            .await
            .map_err(map_storage)?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().await.map_err(map_storage)? {
            out.push(row_to_variant(&row)?);
        }
        Ok(out)
    }
}
