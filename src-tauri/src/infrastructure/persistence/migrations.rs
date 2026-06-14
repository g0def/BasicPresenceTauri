use libsql::Connection;

use crate::domain::error::DomainError;
use crate::infrastructure::persistence::db::map_storage;

pub struct Migration {
    pub version: i64,
    pub sql: &'static str,
}

pub const KEYSTORE_MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: include_str!("../../../migrations/keystore/0001_init.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("../../../migrations/keystore/0002_add_mac_key.sql"),
    },
];

pub const VAULT_MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: include_str!("../../../migrations/vault/0001_init.sql"),
    },
    Migration {
        version: 2,
        sql: include_str!("../../../migrations/vault/0002_add_profiles.sql"),
    },
    Migration {
        version: 3,
        sql: include_str!("../../../migrations/vault/0003_add_presence.sql"),
    },
    Migration {
        version: 4,
        sql: include_str!("../../../migrations/vault/0004_add_co2.sql"),
    },
    Migration {
        version: 5,
        sql: include_str!("../../../migrations/vault/0005_add_work_hours.sql"),
    },
    Migration {
        version: 6,
        sql: include_str!("../../../migrations/vault/0006_add_work_schedule.sql"),
    },
    Migration {
        version: 7,
        sql: include_str!("../../../migrations/vault/0007_simplify_work_schedule.sql"),
    },
    Migration {
        version: 8,
        sql: include_str!("../../../migrations/vault/0008_store_end_minutes.sql"),
    },
    Migration {
        version: 9,
        sql: include_str!("../../../migrations/vault/0009_add_presence_note.sql"),
    },
];

/// Idempotently apply migrations in version order, tracking applied versions in
/// a `_migrations` table on the given connection.
pub async fn run(conn: &Connection, migrations: &[Migration]) -> Result<(), DomainError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL)",
        (),
    )
    .await
    .map_err(map_storage)?;

    // Scoped so the SELECT cursor is closed before any DDL runs: an open
    // statement makes schema-changing migrations (DROP TABLE…) fail with
    // "database table is locked".
    let current: i64 = {
        let mut rows = conn
            .query("SELECT COALESCE(MAX(version), 0) FROM _migrations", ())
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => row.get(0).map_err(map_storage)?,
            None => 0,
        }
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
