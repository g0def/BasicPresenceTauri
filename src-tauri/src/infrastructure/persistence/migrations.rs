use libsql::Connection;

use crate::domain::error::DomainError;
use crate::infrastructure::persistence::db::map_storage;

pub struct Migration {
    pub version: i64,
    pub sql: &'static str,
}

pub const KEYSTORE_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    sql: include_str!("../../../migrations/keystore/0001_init.sql"),
}];

pub const VAULT_MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    sql: include_str!("../../../migrations/vault/0001_init.sql"),
}];

/// Idempotently apply migrations in version order, tracking applied versions in
/// a `_migrations` table on the given connection.
pub async fn run(conn: &Connection, migrations: &[Migration]) -> Result<(), DomainError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL)",
        (),
    )
    .await
    .map_err(map_storage)?;

    let mut rows = conn
        .query("SELECT COALESCE(MAX(version), 0) FROM _migrations", ())
        .await
        .map_err(map_storage)?;
    let current: i64 = match rows.next().await.map_err(map_storage)? {
        Some(row) => row.get(0).map_err(map_storage)?,
        None => 0,
    };

    for m in migrations {
        if m.version > current {
            conn.execute_batch(m.sql).await.map_err(map_storage)?;
            conn.execute(
                "INSERT INTO _migrations (version, applied_at) \
                 VALUES (?1, CAST(strftime('%s','now') AS INTEGER) * 1000)",
                libsql::params![m.version],
            )
            .await
            .map_err(map_storage)?;
        }
    }
    Ok(())
}
