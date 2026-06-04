use std::sync::Arc;

use async_trait::async_trait;
use libsql::{params, Connection, Row};

use crate::domain::entities::profile::Profile;
use crate::domain::error::DomainError;
use crate::domain::repositories::profile_repository::ProfileRepository;
use crate::infrastructure::persistence::db::map_storage;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;

const SELECT_COLUMNS: &str = "id, first_name, last_name, enterprise, poste, created_at, updated_at";

const ACTIVE_KEY: &str = "active_profile_id";

/// Profile repository backed by the (unlocked) encrypted vault connection.
/// Resolves the live connection on every call so a locked vault yields
/// `DomainError::Unauthorized` rather than a stale handle.
pub struct LibsqlProfileRepository {
    vault: Arc<LibsqlVaultManager>,
}

impl LibsqlProfileRepository {
    pub fn new(vault: Arc<LibsqlVaultManager>) -> Self {
        Self { vault }
    }

    fn conn(&self) -> Result<Connection, DomainError> {
        self.vault.connection().ok_or(DomainError::Unauthorized)
    }
}

fn row_to_profile(row: &Row) -> Result<Profile, DomainError> {
    Ok(Profile {
        id: row.get(0).map_err(map_storage)?,
        first_name: row.get(1).map_err(map_storage)?,
        last_name: row.get(2).map_err(map_storage)?,
        enterprise: row.get(3).map_err(map_storage)?,
        poste: row.get(4).map_err(map_storage)?,
        created_at: row.get(5).map_err(map_storage)?,
        updated_at: row.get(6).map_err(map_storage)?,
    })
}

#[async_trait]
impl ProfileRepository for LibsqlProfileRepository {
    async fn create(&self, profile: &Profile) -> Result<(), DomainError> {
        self.conn()?
            .execute(
                "INSERT INTO profile \
                 (id, first_name, last_name, enterprise, poste, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    profile.id.clone(),
                    profile.first_name.clone(),
                    profile.last_name.clone(),
                    profile.enterprise.clone(),
                    profile.poste.clone(),
                    profile.created_at,
                    profile.updated_at,
                ],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Profile>, DomainError> {
        let sql = format!("SELECT {SELECT_COLUMNS} FROM profile ORDER BY created_at");
        let mut rows = self.conn()?.query(&sql, ()).await.map_err(map_storage)?;
        let mut profiles = Vec::new();
        while let Some(row) = rows.next().await.map_err(map_storage)? {
            profiles.push(row_to_profile(&row)?);
        }
        Ok(profiles)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Profile>, DomainError> {
        let sql = format!("SELECT {SELECT_COLUMNS} FROM profile WHERE id = ?1");
        let mut rows = self
            .conn()?
            .query(&sql, params![id])
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => Ok(Some(row_to_profile(&row)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, profile: &Profile) -> Result<(), DomainError> {
        self.conn()?
            .execute(
                "UPDATE profile SET first_name = ?1, last_name = ?2, enterprise = ?3, \
                 poste = ?4, updated_at = ?5 WHERE id = ?6",
                params![
                    profile.first_name.clone(),
                    profile.last_name.clone(),
                    profile.enterprise.clone(),
                    profile.poste.clone(),
                    profile.updated_at,
                    profile.id.clone(),
                ],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        self.conn()?
            .execute("DELETE FROM profile WHERE id = ?1", params![id])
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    async fn active_id(&self) -> Result<Option<String>, DomainError> {
        let mut rows = self
            .conn()?
            .query(
                "SELECT value FROM vault_meta WHERE key = ?1",
                params![ACTIVE_KEY],
            )
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => Ok(Some(row.get(0).map_err(map_storage)?)),
            None => Ok(None),
        }
    }

    async fn set_active(&self, id: Option<&str>) -> Result<(), DomainError> {
        let conn = self.conn()?;
        match id {
            Some(id) => {
                conn.execute(
                    "INSERT INTO vault_meta (key, value) VALUES (?1, ?2) \
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![ACTIVE_KEY, id],
                )
                .await
                .map_err(map_storage)?;
            }
            None => {
                conn.execute("DELETE FROM vault_meta WHERE key = ?1", params![ACTIVE_KEY])
                    .await
                    .map_err(map_storage)?;
            }
        }
        Ok(())
    }
}
