use std::sync::Arc;

use async_trait::async_trait;
use libsql::{params, Connection, Row};

use crate::domain::entities::work_day_schedule::WorkDaySchedule;
use crate::domain::entities::work_entry::WorkEntry;
use crate::domain::error::DomainError;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::infrastructure::persistence::db::map_storage;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;

const ENTRY_COLUMNS: &str = "id, presence_id, title, description, minutes, color, position";

/// Per-day work entries backed by the unlocked vault connection. A day's entry
/// set is always written atomically as a whole (replace-all).
pub struct LibsqlWorkEntryRepository {
    vault: Arc<LibsqlVaultManager>,
}

impl LibsqlWorkEntryRepository {
    pub fn new(vault: Arc<LibsqlVaultManager>) -> Self {
        Self { vault }
    }

    fn conn(&self) -> Result<Connection, DomainError> {
        self.vault.connection().ok_or(DomainError::Unauthorized)
    }
}

fn row_to_entry(row: &Row) -> Result<WorkEntry, DomainError> {
    Ok(WorkEntry {
        id: row.get(0).map_err(map_storage)?,
        presence_id: row.get(1).map_err(map_storage)?,
        title: row.get(2).map_err(map_storage)?,
        description: row.get(3).map_err(map_storage)?,
        minutes: row.get(4).map_err(map_storage)?,
        color: row.get(5).map_err(map_storage)?,
        position: row.get(6).map_err(map_storage)?,
    })
}

#[async_trait]
impl WorkEntryRepository for LibsqlWorkEntryRepository {
    async fn list_by_presence(&self, presence_id: &str) -> Result<Vec<WorkEntry>, DomainError> {
        let sql = format!(
            "SELECT {ENTRY_COLUMNS} FROM work_entry WHERE presence_id = ?1 ORDER BY position"
        );
        let mut rows = self
            .conn()?
            .query(&sql, params![presence_id])
            .await
            .map_err(map_storage)?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next().await.map_err(map_storage)? {
            entries.push(row_to_entry(&row)?);
        }
        Ok(entries)
    }

    async fn replace_for_presence(
        &self,
        presence_id: &str,
        entries: &[WorkEntry],
    ) -> Result<(), DomainError> {
        let conn = self.conn()?;
        conn.execute("BEGIN", ()).await.map_err(map_storage)?;
        let result = async {
            conn.execute(
                "DELETE FROM work_entry WHERE presence_id = ?1",
                params![presence_id],
            )
            .await
            .map_err(map_storage)?;
            let sql = format!(
                "INSERT INTO work_entry ({ENTRY_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
            );
            for e in entries {
                conn.execute(
                    &sql,
                    params![
                        e.id.clone(),
                        e.presence_id.clone(),
                        e.title.clone(),
                        e.description.clone(),
                        e.minutes,
                        e.color.clone(),
                        e.position,
                    ],
                )
                .await
                .map_err(map_storage)?;
            }
            Ok(())
        }
        .await;
        match result {
            Ok(()) => {
                conn.execute("COMMIT", ()).await.map_err(map_storage)?;
                Ok(())
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK", ()).await;
                Err(e)
            }
        }
    }

    async fn get_schedule(
        &self,
        presence_id: &str,
    ) -> Result<Option<WorkDaySchedule>, DomainError> {
        let mut rows = self
            .conn()?
            .query(
                "SELECT presence_id, start_minutes, end_minutes \
                 FROM work_day_schedule WHERE presence_id = ?1",
                params![presence_id],
            )
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => Ok(Some(WorkDaySchedule {
                presence_id: row.get(0).map_err(map_storage)?,
                start_minutes: row.get(1).map_err(map_storage)?,
                end_minutes: row.get(2).map_err(map_storage)?,
            })),
            None => Ok(None),
        }
    }

    async fn set_schedule(&self, schedule: &WorkDaySchedule) -> Result<(), DomainError> {
        self.conn()?
            .execute(
                "INSERT INTO work_day_schedule (presence_id, start_minutes, end_minutes) \
                 VALUES (?1, ?2, ?3) \
                 ON CONFLICT(presence_id) DO UPDATE SET \
                   start_minutes = excluded.start_minutes, \
                   end_minutes = excluded.end_minutes",
                params![
                    schedule.presence_id.clone(),
                    schedule.start_minutes,
                    schedule.end_minutes,
                ],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    async fn clear_schedule(&self, presence_id: &str) -> Result<(), DomainError> {
        self.conn()?
            .execute(
                "DELETE FROM work_day_schedule WHERE presence_id = ?1",
                params![presence_id],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }
}
