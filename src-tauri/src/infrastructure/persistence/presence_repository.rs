use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use libsql::{params, Connection, Row};
use uuid::Uuid;

use crate::domain::entities::presence::{Presence, PresenceType};
use crate::domain::entities::trip::Trip;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::{ImportCounts, PresenceRepository};
use crate::infrastructure::persistence::db::map_storage;
use crate::infrastructure::persistence::vault::LibsqlVaultManager;

const SELECT_COLUMNS: &str =
    "id, profile_id, day, type, co2_kg, is_estimated, created_at, updated_at, (SELECT COALESCE(SUM(minutes), 0) FROM work_entry WHERE presence_id = presence.id) AS work_minutes";

const RETURNING_COLUMNS: &str =
    "id, profile_id, day, type, co2_kg, is_estimated, created_at, updated_at";

/// Columns written on insert (includes the owning `presence_id`).
const TRIP_INSERT_COLUMNS: &str =
    "id, presence_id, mode_id, distance_km, round_trip, occupants, co2_kg, is_estimated, factor_year, position";
/// Columns read back (the `presence_id` is implied by the query filter).
const TRIP_SELECT_COLUMNS: &str =
    "id, mode_id, distance_km, round_trip, occupants, co2_kg, is_estimated, factor_year, position";

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
    let is_estimated: i64 = row.get(5).map_err(map_storage)?;
    Ok(Presence {
        id: row.get(0).map_err(map_storage)?,
        profile_id: row.get(1).map_err(map_storage)?,
        day: row.get(2).map_err(map_storage)?,
        kind: PresenceType::parse(&kind)?,
        co2_kg: row.get(4).map_err(map_storage)?,
        is_estimated: is_estimated != 0,
        created_at: row.get(6).map_err(map_storage)?,
        updated_at: row.get(7).map_err(map_storage)?,
        work_minutes: row.get(8).map_err(map_storage)?,
    })
}

fn row_to_presence_returning(row: &Row) -> Result<Presence, DomainError> {
    let kind: String = row.get(3).map_err(map_storage)?;
    let is_estimated: i64 = row.get(5).map_err(map_storage)?;
    Ok(Presence {
        id: row.get(0).map_err(map_storage)?,
        profile_id: row.get(1).map_err(map_storage)?,
        day: row.get(2).map_err(map_storage)?,
        kind: PresenceType::parse(&kind)?,
        co2_kg: row.get(4).map_err(map_storage)?,
        is_estimated: is_estimated != 0,
        created_at: row.get(6).map_err(map_storage)?,
        updated_at: row.get(7).map_err(map_storage)?,
        work_minutes: 0,
    })
}

fn row_to_trip(row: &Row) -> Result<Trip, DomainError> {
    let round_trip: i64 = row.get(3).map_err(map_storage)?;
    let is_estimated: i64 = row.get(6).map_err(map_storage)?;
    let factor_year: i64 = row.get(7).map_err(map_storage)?;
    Ok(Trip {
        id: row.get(0).map_err(map_storage)?,
        mode_id: row.get(1).map_err(map_storage)?,
        distance_km: row.get(2).map_err(map_storage)?,
        round_trip: round_trip != 0,
        occupants: row.get(4).map_err(map_storage)?,
        co2_kg: row.get(5).map_err(map_storage)?,
        is_estimated: is_estimated != 0,
        factor_year: factor_year as i32,
        position: row.get(8).map_err(map_storage)?,
    })
}

#[async_trait]
impl PresenceRepository for LibsqlPresenceRepository {
    async fn set_for_day(
        &self,
        presence: &Presence,
        trips: &[Trip],
    ) -> Result<Presence, DomainError> {
        let conn = self.conn()?;
        // The presence upsert and its trip snapshot must commit together, so the
        // day total and its breakdown can never diverge.
        conn.execute("BEGIN", ()).await.map_err(map_storage)?;
        match set_for_day_in_tx(&conn, presence, trips).await {
            Ok(saved) => {
                conn.execute("COMMIT", ()).await.map_err(map_storage)?;
                Ok(saved)
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK", ()).await;
                Err(e)
            }
        }
    }

    async fn list_trips(&self, presence_id: &str) -> Result<Vec<Trip>, DomainError> {
        let sql = format!(
            "SELECT {TRIP_SELECT_COLUMNS} FROM presence_trip WHERE presence_id = ?1 ORDER BY position"
        );
        let mut rows = self
            .conn()?
            .query(&sql, params![presence_id])
            .await
            .map_err(map_storage)?;
        let mut trips = Vec::new();
        while let Some(row) = rows.next().await.map_err(map_storage)? {
            trips.push(row_to_trip(&row)?);
        }
        Ok(trips)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Presence>, DomainError> {
        let sql = format!("SELECT {SELECT_COLUMNS} FROM presence WHERE id = ?1");
        let mut rows = self
            .conn()?
            .query(&sql, params![id])
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => Ok(Some(row_to_presence(&row)?)),
            None => Ok(None),
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

/// Upsert the presence and replace its trip snapshot, inside an open transaction.
async fn set_for_day_in_tx(
    conn: &Connection,
    presence: &Presence,
    trips: &[Trip],
) -> Result<Presence, DomainError> {
    // Upsert keyed on (profile_id, day): on conflict we refresh the type, the
    // day total and updated_at, preserving the original id/created_at. RETURNING
    // hands back the persisted row (whose id we reuse for the trip rows).
    let sql = format!(
        "INSERT INTO presence (id, profile_id, day, type, co2_kg, is_estimated, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
         ON CONFLICT(profile_id, day) DO UPDATE SET \
         type = excluded.type, co2_kg = excluded.co2_kg, \
         is_estimated = excluded.is_estimated, updated_at = excluded.updated_at \
         RETURNING {RETURNING_COLUMNS}"
    );
    let mut saved = {
        let mut rows = conn
            .query(
                &sql,
                params![
                    presence.id.clone(),
                    presence.profile_id.clone(),
                    presence.day,
                    presence.kind.as_str(),
                    presence.co2_kg,
                    presence.is_estimated as i64,
                    presence.created_at,
                    presence.updated_at,
                ],
            )
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => row_to_presence_returning(&row)?,
            None => {
                return Err(DomainError::Storage(
                    "presence upsert returned no row".into(),
                ))
            }
        }
    };

    // Replace-all the trip snapshot for this presence.
    conn.execute(
        "DELETE FROM presence_trip WHERE presence_id = ?1",
        params![saved.id.clone()],
    )
    .await
    .map_err(map_storage)?;

    let insert = format!(
        "INSERT INTO presence_trip ({TRIP_INSERT_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
    );
    for t in trips {
        conn.execute(
            &insert,
            params![
                Uuid::now_v7().to_string(),
                saved.id.clone(),
                t.mode_id.clone(),
                t.distance_km,
                t.round_trip as i64,
                t.occupants,
                t.co2_kg,
                t.is_estimated as i64,
                i64::from(t.factor_year),
                t.position,
            ],
        )
        .await
        .map_err(map_storage)?;
    }

    // Since the type might have changed or we might have updated the presence,
    // fetch the work_minutes from the database.
    let work_minutes = {
        let mut rows = conn
            .query(
                "SELECT COALESCE(SUM(minutes), 0) FROM work_entry WHERE presence_id = ?1",
                params![saved.id.clone()],
            )
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => row.get::<i64>(0).map_err(map_storage)?,
            None => 0,
        }
    };
    saved.work_minutes = work_minutes;

    Ok(saved)
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
