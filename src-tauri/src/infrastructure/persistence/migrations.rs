use libsql::Connection;

use crate::domain::error::DomainError;
use crate::infrastructure::persistence::db::{commit_or_rollback, map_storage};

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
    Migration {
        version: 3,
        sql: include_str!("../../../migrations/keystore/0003_add_vault_baseline.sql"),
    },
    Migration {
        version: 4,
        sql: include_str!("../../../migrations/keystore/0004_persist_kek_params.sql"),
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
    Migration {
        version: 10,
        sql: include_str!("../../../migrations/vault/0010_add_profile_settings.sql"),
    },
    Migration {
        version: 11,
        sql: include_str!("../../../migrations/vault/0011_add_co2_2026.sql"),
    },
    Migration {
        version: 12,
        sql: include_str!("../../../migrations/vault/0012_guard_factor_year.sql"),
    },
];

/// Apply migrations in version order, tracking applied versions in a
/// `_migrations` table on the given connection.
///
/// Each migration is applied **atomically with the recording of its version**:
/// a crash can therefore never leave the schema changed while `_migrations`
/// still reports the older version. That guarantee matters because several
/// migrations are not replayable — SQLite has no `ADD COLUMN IF NOT EXISTS`, so
/// replaying `0004`, `0008`, `0009` (or the `DROP TABLE` of `0007`) would fail
/// forever and permanently wedge the database.
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

    // Refuse to touch a database written by a newer build: its schema may hold
    // columns and constraints this binary knows nothing about, and writing to it
    // would silently corrupt data the newer version depends on. Downgrading the
    // app must fail loudly rather than degrade the store.
    let known = migrations.iter().map(|m| m.version).max().unwrap_or(0);
    if current > known {
        return Err(DomainError::SchemaTooNew {
            found: current,
            supported: known,
        });
    }

    for m in migrations {
        if m.version > current {
            apply_atomically(conn, m).await?;
        }
    }
    Ok(())
}

/// Run one migration's SQL and record its version inside a single transaction,
/// rolling back both on any failure. SQLite supports transactional DDL, so the
/// `ALTER TABLE` / `DROP TABLE` statements are covered too.
async fn apply_atomically(conn: &Connection, m: &Migration) -> Result<(), DomainError> {
    conn.execute("BEGIN", ()).await.map_err(map_storage)?;
    match apply_in_tx(conn, m).await {
        Ok(()) => {
            commit_or_rollback(conn).await?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            Err(e)
        }
    }
}

async fn apply_in_tx(conn: &Connection, m: &Migration) -> Result<(), DomainError> {
    conn.execute_batch(m.sql).await.map_err(map_storage)?;
    conn.execute(
        "INSERT INTO _migrations (version, applied_at) \
         VALUES (?1, CAST(strftime('%s','now') AS INTEGER) * 1000)",
        libsql::params![m.version],
    )
    .await
    .map_err(map_storage)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::persistence::db::{connect, open_plain_db};

    async fn memory_conn() -> Connection {
        let db = open_plain_db(std::path::Path::new(":memory:"))
            .await
            .expect("open in-memory db");
        connect(&db).await.expect("connect")
    }

    async fn applied_versions(conn: &Connection) -> Vec<i64> {
        let mut rows = conn
            .query("SELECT version FROM _migrations ORDER BY version", ())
            .await
            .expect("query");
        let mut out = Vec::new();
        while let Some(row) = rows.next().await.expect("row") {
            out.push(row.get(0).expect("version"));
        }
        out
    }

    async fn table_exists(conn: &Connection, name: &str) -> bool {
        let mut rows = conn
            .query(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                libsql::params![name],
            )
            .await
            .expect("query");
        rows.next().await.expect("row").is_some()
    }

    /// A migration that fails halfway must leave *nothing* behind: neither the
    /// half-applied schema nor a recorded version. Without the transaction, the
    /// first statement would stick, the version would be missing, and the replay
    /// would hit "table already exists" on every subsequent launch.
    #[test]
    fn a_failing_migration_leaves_no_trace() {
        tauri::async_runtime::block_on(async {
            let conn = memory_conn().await;
            let broken = [Migration {
                version: 1,
                sql: "CREATE TABLE early (id TEXT PRIMARY KEY);\n\
                      THIS IS NOT VALID SQL;",
            }];

            let err = run(&conn, &broken).await.expect_err("migration must fail");
            assert!(matches!(err, DomainError::Storage(_)));

            assert!(
                !table_exists(&conn, "early").await,
                "the first statement must have been rolled back"
            );
            assert!(
                applied_versions(&conn).await.is_empty(),
                "no version may be recorded for a failed migration"
            );
        });
    }

    /// After a failed attempt, a corrected build must be able to replay the same
    /// version from a clean slate.
    #[test]
    fn a_failed_migration_can_be_replayed_once_fixed() {
        tauri::async_runtime::block_on(async {
            let conn = memory_conn().await;
            let broken = [Migration {
                version: 1,
                sql: "CREATE TABLE thing (id TEXT PRIMARY KEY);\nBOOM;",
            }];
            assert!(run(&conn, &broken).await.is_err());

            let fixed = [Migration {
                version: 1,
                sql: "CREATE TABLE thing (id TEXT PRIMARY KEY);",
            }];
            run(&conn, &fixed).await.expect("replay must succeed");

            assert!(table_exists(&conn, "thing").await);
            assert_eq!(applied_versions(&conn).await, vec![1]);
        });
    }

    /// The version is recorded in the same transaction as the schema change, so
    /// the two can never disagree.
    #[test]
    fn schema_and_version_commit_together() {
        tauri::async_runtime::block_on(async {
            let conn = memory_conn().await;
            let set = [
                Migration {
                    version: 1,
                    sql: "CREATE TABLE a (id TEXT PRIMARY KEY);",
                },
                Migration {
                    version: 2,
                    sql: "ALTER TABLE a ADD COLUMN note TEXT;",
                },
            ];
            run(&conn, &set).await.expect("migrations");
            assert_eq!(applied_versions(&conn).await, vec![1, 2]);

            // Re-running is a no-op: the non-idempotent ALTER is not replayed.
            run(&conn, &set).await.expect("second run is a no-op");
            assert_eq!(applied_versions(&conn).await, vec![1, 2]);
        });
    }

    /// A database written by a newer build must be refused, not silently used.
    #[test]
    fn a_newer_schema_is_refused() {
        tauri::async_runtime::block_on(async {
            let conn = memory_conn().await;
            let future = [
                Migration {
                    version: 1,
                    sql: "CREATE TABLE a (id TEXT PRIMARY KEY);",
                },
                Migration {
                    version: 2,
                    sql: "CREATE TABLE b (id TEXT PRIMARY KEY);",
                },
            ];
            run(&conn, &future).await.expect("migrations");

            // Same store, opened by an older binary that only knows version 1.
            let older = [Migration {
                version: 1,
                sql: "CREATE TABLE a (id TEXT PRIMARY KEY);",
            }];
            let err = run(&conn, &older).await.expect_err("must refuse");
            assert!(matches!(
                err,
                DomainError::SchemaTooNew {
                    found: 2,
                    supported: 1
                }
            ));
        });
    }
}
