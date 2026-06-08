use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use libsql::{params, Connection, Row};

use crate::domain::entities::commute::{Commute, CommuteSegment};
use crate::domain::error::DomainError;
use crate::domain::repositories::commute_repository::CommuteRepository;
use crate::infrastructure::persistence::db::map_storage;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;

const COMMUTE_COLUMNS: &str = "id, profile_id, name, round_trip, created_at, updated_at";
const SEGMENT_COLUMNS: &str = "id, commute_id, mode_id, distance_km, occupants, position";

/// Saved commute templates backed by the unlocked vault connection. A commute
/// and its ordered segments are written atomically.
pub struct LibsqlCommuteRepository {
    vault: Arc<LibsqlVaultManager>,
}

impl LibsqlCommuteRepository {
    pub fn new(vault: Arc<LibsqlVaultManager>) -> Self {
        Self { vault }
    }

    fn conn(&self) -> Result<Connection, DomainError> {
        self.vault.connection().ok_or(DomainError::Unauthorized)
    }
}

fn row_to_commute(row: &Row) -> Result<Commute, DomainError> {
    let round_trip: i64 = row.get(3).map_err(map_storage)?;
    Ok(Commute {
        id: row.get(0).map_err(map_storage)?,
        profile_id: row.get(1).map_err(map_storage)?,
        name: row.get(2).map_err(map_storage)?,
        round_trip: round_trip != 0,
        segments: Vec::new(),
        created_at: row.get(4).map_err(map_storage)?,
        updated_at: row.get(5).map_err(map_storage)?,
    })
}

fn row_to_segment(row: &Row) -> Result<CommuteSegment, DomainError> {
    Ok(CommuteSegment {
        id: row.get(0).map_err(map_storage)?,
        commute_id: row.get(1).map_err(map_storage)?,
        mode_id: row.get(2).map_err(map_storage)?,
        distance_km: row.get(3).map_err(map_storage)?,
        occupants: row.get(4).map_err(map_storage)?,
        position: row.get(5).map_err(map_storage)?,
    })
}

async fn insert_segments(conn: &Connection, commute: &Commute) -> Result<(), DomainError> {
    let sql =
        format!("INSERT INTO commute_segment ({SEGMENT_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6)");
    for s in &commute.segments {
        conn.execute(
            &sql,
            params![
                s.id.clone(),
                commute.id.clone(),
                s.mode_id.clone(),
                s.distance_km,
                s.occupants,
                s.position,
            ],
        )
        .await
        .map_err(map_storage)?;
    }
    Ok(())
}

#[async_trait]
impl CommuteRepository for LibsqlCommuteRepository {
    async fn create(&self, commute: &Commute) -> Result<(), DomainError> {
        let conn = self.conn()?;
        conn.execute("BEGIN", ()).await.map_err(map_storage)?;
        let result = async {
            conn.execute(
                &format!("INSERT INTO commute ({COMMUTE_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"),
                params![
                    commute.id.clone(),
                    commute.profile_id.clone(),
                    commute.name.clone(),
                    commute.round_trip as i64,
                    commute.created_at,
                    commute.updated_at,
                ],
            )
            .await
            .map_err(map_storage)?;
            insert_segments(&conn, commute).await
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

    async fn list_by_profile(&self, profile_id: &str) -> Result<Vec<Commute>, DomainError> {
        let conn = self.conn()?;
        let mut commutes: Vec<Commute> = {
            let sql = format!(
                "SELECT {COMMUTE_COLUMNS} FROM commute WHERE profile_id = ?1 ORDER BY name"
            );
            let mut rows = conn
                .query(&sql, params![profile_id])
                .await
                .map_err(map_storage)?;
            let mut out = Vec::new();
            while let Some(row) = rows.next().await.map_err(map_storage)? {
                out.push(row_to_commute(&row)?);
            }
            out
        };

        // Load every segment of the profile's commutes in one query, then group.
        let mut by_commute: HashMap<String, Vec<CommuteSegment>> = HashMap::new();
        {
            let sql =
                "SELECT s.id, s.commute_id, s.mode_id, s.distance_km, s.occupants, s.position \
                 FROM commute_segment s JOIN commute c ON c.id = s.commute_id \
                 WHERE c.profile_id = ?1 ORDER BY s.commute_id, s.position";
            let mut rows = conn
                .query(sql, params![profile_id])
                .await
                .map_err(map_storage)?;
            while let Some(row) = rows.next().await.map_err(map_storage)? {
                let seg = row_to_segment(&row)?;
                by_commute
                    .entry(seg.commute_id.clone())
                    .or_default()
                    .push(seg);
            }
        }
        for c in &mut commutes {
            if let Some(segs) = by_commute.remove(&c.id) {
                c.segments = segs;
            }
        }
        Ok(commutes)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Commute>, DomainError> {
        let conn = self.conn()?;
        let sql = format!("SELECT {COMMUTE_COLUMNS} FROM commute WHERE id = ?1");
        let mut rows = conn.query(&sql, params![id]).await.map_err(map_storage)?;
        let mut commute = match rows.next().await.map_err(map_storage)? {
            Some(row) => row_to_commute(&row)?,
            None => return Ok(None),
        };

        let seg_sql = format!(
            "SELECT {SEGMENT_COLUMNS} FROM commute_segment WHERE commute_id = ?1 ORDER BY position"
        );
        let mut seg_rows = conn
            .query(&seg_sql, params![id])
            .await
            .map_err(map_storage)?;
        while let Some(row) = seg_rows.next().await.map_err(map_storage)? {
            commute.segments.push(row_to_segment(&row)?);
        }
        Ok(Some(commute))
    }

    async fn update(&self, commute: &Commute) -> Result<(), DomainError> {
        let conn = self.conn()?;
        conn.execute("BEGIN", ()).await.map_err(map_storage)?;
        let result = async {
            conn.execute(
                "UPDATE commute SET name = ?1, round_trip = ?2, updated_at = ?3 WHERE id = ?4",
                params![
                    commute.name.clone(),
                    commute.round_trip as i64,
                    commute.updated_at,
                    commute.id.clone(),
                ],
            )
            .await
            .map_err(map_storage)?;
            // Replace-all: segment ids are not surfaced as stable handles.
            conn.execute(
                "DELETE FROM commute_segment WHERE commute_id = ?1",
                params![commute.id.clone()],
            )
            .await
            .map_err(map_storage)?;
            insert_segments(&conn, commute).await
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

    async fn delete(&self, id: &str) -> Result<(), DomainError> {
        // Segments cascade via the FK (PRAGMA foreign_keys = ON per connection).
        self.conn()?
            .execute("DELETE FROM commute WHERE id = ?1", params![id])
            .await
            .map_err(map_storage)?;
        Ok(())
    }
}
