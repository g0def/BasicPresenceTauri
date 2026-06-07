use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use libsql::{params, Connection, Row};

use crate::domain::entities::presence::{Presence, PresenceType};
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::{ImportCounts, PresenceRepository};
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

    async fn import_many(
        &self,
        profile_id: &str,
        entries: &[Presence],
        replace_existing: bool,
    ) -> Result<ImportCounts, DomainError> {
        let conn = self.conn()?;

        // Wrap the whole batch in a transaction so the import is atomic: a
        // failure on any row rolls everything back rather than leaving a
        // half-imported vault.
        conn.execute("BEGIN", ()).await.map_err(map_storage)?;
        match import_in_tx(&conn, profile_id, entries, replace_existing).await {
            Ok(counts) => {
                conn.execute("COMMIT", ()).await.map_err(map_storage)?;
                Ok(counts)
            }
            Err(e) => {
                // Best-effort rollback; surface the original error.
                let _ = conn.execute("ROLLBACK", ()).await;
                Err(e)
            }
        }
    }
}

/// Body of [`LibsqlPresenceRepository::import_many`], run inside an open
/// transaction so the caller can COMMIT/ROLLBACK around it.
async fn import_in_tx(
    conn: &Connection,
    profile_id: &str,
    entries: &[Presence],
    replace_existing: bool,
) -> Result<ImportCounts, DomainError> {
    // Snapshot the profile's existing days up front: under `ON CONFLICT DO
    // UPDATE`, `rows_affected()` is 1 for both a fresh insert and an update, so
    // it can't tell them apart. The set also makes duplicate days *within* the
    // same import deterministic (a repeated day is treated as a conflict).
    let mut existing: HashSet<i64> = HashSet::new();
    {
        let mut rows = conn
            .query(
                "SELECT day FROM presence WHERE profile_id = ?1",
                params![profile_id],
            )
            .await
            .map_err(map_storage)?;
        while let Some(row) = rows.next().await.map_err(map_storage)? {
            let day: i64 = row.get(0).map_err(map_storage)?;
            existing.insert(day);
        }
    }

    let sql = if replace_existing {
        "INSERT INTO presence (id, profile_id, day, type, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT(profile_id, day) DO UPDATE SET \
         type = excluded.type, updated_at = excluded.updated_at"
    } else {
        "INSERT INTO presence (id, profile_id, day, type, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT(profile_id, day) DO NOTHING"
    };

    let mut counts = ImportCounts::default();
    for p in entries {
        let conflicts = existing.contains(&p.day);
        conn.execute(
            sql,
            params![
                p.id.clone(),
                p.profile_id.clone(),
                p.day,
                p.kind.as_str(),
                p.created_at,
                p.updated_at,
            ],
        )
        .await
        .map_err(map_storage)?;

        if conflicts {
            // `replace` updated the existing row; `skip` did nothing (the use
            // case derives `skipped = total - inserted - updated`).
            if replace_existing {
                counts.updated += 1;
            }
        } else {
            counts.inserted += 1;
            existing.insert(p.day);
        }
    }

    Ok(counts)
}
