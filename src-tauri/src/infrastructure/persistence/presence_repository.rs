use std::sync::Arc;

use async_trait::async_trait;
use libsql::{params, Connection, Row};

use crate::domain::entities::presence::{Presence, PresenceType};
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::infrastructure::persistence::db::map_storage;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;

const SELECT_COLUMNS: &str = "id, profile_id, day, type, created_at, updated_at";

/// Presence repository backed by the (unlocked) encrypted vault connection.
/// Resolves the live connection on every call so a locked vault yields
/// `DomainError::Unauthorized` rather than a stale handle.
pub struct LibsqlPresenceRepository {
    vault: Arc<LibsqlVaultManager>,
}

impl LibsqlPresenceRepository {
    pub fn new(vault: Arc<LibsqlVaultManager>) -> Self {
        Self { vault }
    }

    fn conn(&self) -> Result<Connection, DomainError> {
        self.vault.connection().ok_or(DomainError::Unauthorized)
    }
}

fn row_to_presence(row: &Row) -> Result<Presence, DomainError> {
    let kind: String = row.get(3).map_err(map_storage)?;
    Ok(Presence {
        id: row.get(0).map_err(map_storage)?,
        profile_id: row.get(1).map_err(map_storage)?,
        day: row.get(2).map_err(map_storage)?,
        kind: PresenceType::parse(&kind)?,
        created_at: row.get(4).map_err(map_storage)?,
        updated_at: row.get(5).map_err(map_storage)?,
    })
}

#[async_trait]
impl PresenceRepository for LibsqlPresenceRepository {
    async fn set_for_day(&self, presence: &Presence) -> Result<Presence, DomainError> {
        // Upsert keyed on (profile_id, day): on conflict we only refresh the
        // type and updated_at, preserving the original id/created_at. RETURNING
        // hands back the persisted row.
        let sql = format!(
            "INSERT INTO presence (id, profile_id, day, type, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(profile_id, day) DO UPDATE SET \
             type = excluded.type, updated_at = excluded.updated_at \
             RETURNING {SELECT_COLUMNS}"
        );
        let mut rows = self
            .conn()?
            .query(
                &sql,
                params![
                    presence.id.clone(),
                    presence.profile_id.clone(),
                    presence.day,
                    presence.kind.as_str(),
                    presence.created_at,
                    presence.updated_at,
                ],
            )
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => row_to_presence(&row),
            None => Err(DomainError::Storage(
                "presence upsert returned no row".into(),
            )),
        }
    }

    async fn list_by_profile(&self, profile_id: &str) -> Result<Vec<Presence>, DomainError> {
        let sql =
            format!("SELECT {SELECT_COLUMNS} FROM presence WHERE profile_id = ?1 ORDER BY day");
        let mut rows = self
            .conn()?
            .query(&sql, params![profile_id])
            .await
            .map_err(map_storage)?;
        let mut presences = Vec::new();
        while let Some(row) = rows.next().await.map_err(map_storage)? {
            presences.push(row_to_presence(&row)?);
        }
        Ok(presences)
    }

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        self.conn()?
            .execute("DELETE FROM presence WHERE id = ?1", params![id])
            .await
            .map_err(map_storage)?;
        Ok(())
    }
}
