use std::sync::Arc;

use async_trait::async_trait;
use libsql::{params, Connection, Row};

use crate::domain::entities::task_preset::TaskPreset;
use crate::domain::error::DomainError;
use crate::domain::repositories::task_preset_repository::TaskPresetRepository;
use crate::infrastructure::persistence::db::map_storage;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;

const PRESET_COLUMNS: &str =
    "id, profile_id, title, description, default_minutes, color, created_at, updated_at";

/// Saved task presets backed by the unlocked vault connection.
pub struct LibsqlTaskPresetRepository {
    vault: Arc<LibsqlVaultManager>,
}

impl LibsqlTaskPresetRepository {
    pub fn new(vault: Arc<LibsqlVaultManager>) -> Self {
        Self { vault }
    }

    fn conn(&self) -> Result<Connection, DomainError> {
        self.vault.connection().ok_or(DomainError::Unauthorized)
    }
}

fn row_to_preset(row: &Row) -> Result<TaskPreset, DomainError> {
    Ok(TaskPreset {
        id: row.get(0).map_err(map_storage)?,
        profile_id: row.get(1).map_err(map_storage)?,
        title: row.get(2).map_err(map_storage)?,
        description: row.get(3).map_err(map_storage)?,
        default_minutes: row.get(4).map_err(map_storage)?,
        color: row.get(5).map_err(map_storage)?,
        created_at: row.get(6).map_err(map_storage)?,
        updated_at: row.get(7).map_err(map_storage)?,
    })
}

#[async_trait]
impl TaskPresetRepository for LibsqlTaskPresetRepository {
    async fn create(&self, preset: &TaskPreset) -> Result<(), DomainError> {
        self.conn()?
            .execute(
                &format!(
                    "INSERT INTO task_preset ({PRESET_COLUMNS}) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
                ),
                params![
                    preset.id.clone(),
                    preset.profile_id.clone(),
                    preset.title.clone(),
                    preset.description.clone(),
                    preset.default_minutes,
                    preset.color.clone(),
                    preset.created_at,
                    preset.updated_at,
                ],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    async fn list_by_profile(&self, profile_id: &str) -> Result<Vec<TaskPreset>, DomainError> {
        let sql = format!(
            "SELECT {PRESET_COLUMNS} FROM task_preset WHERE profile_id = ?1 ORDER BY title"
        );
        let mut rows = self
            .conn()?
            .query(&sql, params![profile_id])
            .await
            .map_err(map_storage)?;
        let mut presets = Vec::new();
        while let Some(row) = rows.next().await.map_err(map_storage)? {
            presets.push(row_to_preset(&row)?);
        }
        Ok(presets)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<TaskPreset>, DomainError> {
        let sql = format!("SELECT {PRESET_COLUMNS} FROM task_preset WHERE id = ?1");
        let mut rows = self
            .conn()?
            .query(&sql, params![id])
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => Ok(Some(row_to_preset(&row)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, preset: &TaskPreset) -> Result<(), DomainError> {
        self.conn()?
            .execute(
                "UPDATE task_preset SET title = ?1, description = ?2, default_minutes = ?3, \
                 color = ?4, updated_at = ?5 WHERE id = ?6",
                params![
                    preset.title.clone(),
                    preset.description.clone(),
                    preset.default_minutes,
                    preset.color.clone(),
                    preset.updated_at,
                    preset.id.clone(),
                ],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        self.conn()?
            .execute("DELETE FROM task_preset WHERE id = ?1", params![id])
            .await
            .map_err(map_storage)?;
        Ok(())
    }
}
