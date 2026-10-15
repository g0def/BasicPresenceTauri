use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use libsql::{Connection, Database};
use zeroize::Zeroizing;

use crate::domain::error::DomainError;
use crate::domain::services::clock::Clock;
use crate::domain::services::unit_of_work::{UnitOfWork, Work};
use crate::domain::services::vault::{VaultManager, VaultOpenOutcome};
use crate::infrastructure::config::IntegrityPolicy;
use crate::infrastructure::persistence::db::{configure, connect, map_storage, open_encrypted_db};
use crate::infrastructure::persistence::integrity_store::KeystoreIntegrityStore;
use crate::infrastructure::persistence::migrations::{self, VAULT_MIGRATIONS};
use crate::infrastructure::persistence::vault_integrity::{self, IntegrityVerdict};

tokio::task_local! {
    /// Connection of the write transaction the current task is running in, if
    /// any. Set by [`LibsqlVaultManager::write`] / [`LibsqlVaultManager::unit_of_work`]
    /// so that every repository call made inside the transaction — reads
    /// included — goes through the transaction's dedicated connection.
    static AMBIENT_TX: Connection;
}

struct OpenVault {
    // Kept alive for the session; each write transaction opens its own
    // connection from it.
    db: Database,
    // Shared connection for reads (autocommit) outside a transaction.
    conn: Connection,
    // MAC key, held only while unlocked, so `close()` can re-baseline the file.
    mac_key: Zeroizing<Vec<u8>>,
}

/// libSQL-backed encrypted vault. Holds an open connection only while unlocked;
/// locking drops the connection (and the in-memory key it carries).
///
/// **Writes.** Every write goes through [`Self::write`], which serializes
/// writers in-process and runs each on a *dedicated* connection inside
/// `BEGIN IMMEDIATE … COMMIT`. Sharing one connection between concurrently
/// running commands would let one command's `BEGIN`/`ROLLBACK` swallow or
/// discard another command's writes.
pub struct LibsqlVaultManager {
    path: PathBuf,
    integrity: IntegrityPolicy,
    integrity_store: KeystoreIntegrityStore,
    clock: Arc<dyn Clock>,
    state: Mutex<Option<OpenVault>>,
    // Held for the whole duration of a write transaction, and by `close()` so a
    // lock never happens mid-write.
    write_lock: Arc<tokio::sync::Mutex<()>>,
}

impl LibsqlVaultManager {
    pub fn new(
        path: PathBuf,
        integrity: IntegrityPolicy,
        integrity_store: KeystoreIntegrityStore,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            path,
            integrity,
            integrity_store,
            clock,
            state: Mutex::new(None),
            write_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    /// Connection for reads: the ambient transaction's when called inside one
    /// (so a unit of work reads its own uncommitted writes), else the shared
    /// connection — `None` while the vault is locked.
    pub fn connection(&self) -> Option<Connection> {
        if let Ok(tx) = AMBIENT_TX.try_with(Connection::clone) {
            return Some(tx);
        }
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .map(|v| v.conn.clone())
    }

    /// Run `work` as (part of) a write transaction.
    ///
    /// Outside a transaction, this takes the write lock, opens a dedicated
    /// connection, and commits if `work` succeeds (rolls back otherwise).
    /// Inside one (a unit of work), `work` simply joins it.
    pub async fn write<T, F, Fut>(&self, work: F) -> Result<T, DomainError>
    where
        F: FnOnce(Connection) -> Fut,
        Fut: Future<Output = Result<T, DomainError>>,
    {
        if let Ok(tx) = AMBIENT_TX.try_with(Connection::clone) {
            return work(tx).await;
        }
        let _permit = self.write_lock.clone().lock_owned().await;
        let conn = self.dedicated_connection().await?;
        run_in_tx(&conn, AMBIENT_TX.scope(conn.clone(), work(conn.clone()))).await
    }

    /// Run several repository calls as one atomic transaction: every write (and
    /// read) they make inside `work` joins the same transaction.
    pub async fn unit_of_work<T, Fut>(&self, work: Fut) -> Result<T, DomainError>
    where
        Fut: Future<Output = Result<T, DomainError>>,
    {
        if AMBIENT_TX.try_with(|_| ()).is_ok() {
            return work.await; // nested: join the enclosing transaction
        }
        let _permit = self.write_lock.clone().lock_owned().await;
        let conn = self.dedicated_connection().await?;
        run_in_tx(&conn, AMBIENT_TX.scope(conn.clone(), work)).await
    }

    /// Single-statement write, as its own (or the ambient) transaction.
    pub async fn execute_write(
        &self,
        sql: &str,
        params: impl libsql::params::IntoParams,
    ) -> Result<u64, DomainError> {
        self.write(|conn| async move { conn.execute(sql, params).await.map_err(map_storage) })
            .await
    }

    async fn dedicated_connection(&self) -> Result<Connection, DomainError> {
        let conn = {
            let guard = self.state.lock().unwrap_or_else(|p| p.into_inner());
            let open = guard.as_ref().ok_or(DomainError::Unauthorized)?;
            open.db.connect().map_err(map_storage)?
        };
        configure(&conn).await?;
        Ok(conn)
    }
}

/// `BEGIN IMMEDIATE` → `work` → `COMMIT`, or `ROLLBACK` if `work` fails.
async fn run_in_tx<T>(
    conn: &Connection,
    work: impl Future<Output = Result<T, DomainError>>,
) -> Result<T, DomainError> {
    conn.execute("BEGIN IMMEDIATE", ())
        .await
        .map_err(map_storage)?;
    match work.await {
        Ok(value) => {
            conn.execute("COMMIT", ()).await.map_err(map_storage)?;
            Ok(value)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            Err(e)
        }
    }
}

#[async_trait]
impl VaultManager for LibsqlVaultManager {
    async fn open(&self, dek: &[u8], mac_key: &[u8]) -> Result<VaultOpenOutcome, DomainError> {
        // Re-opening an already open vault would drop the live state; close it
        // cleanly first so its baseline is recorded.
        self.close().await;

        // Verify at-rest integrity BEFORE opening (the open + migrations will
        // mutate the file), against the keystore-sealed record only.
        let record = self.integrity_store.load().await?;
        let verdict = vault_integrity::assess(&self.path, mac_key, &record)?;
        let outcome = match verdict {
            IntegrityVerdict::Clean | IntegrityVerdict::Fresh | IntegrityVerdict::NoBaseline => {
                VaultOpenOutcome::Clean
            }
            IntegrityVerdict::UncleanShutdown => VaultOpenOutcome::RecoveredFromUncleanShutdown,
            IntegrityVerdict::Mismatch => {
                if self.integrity == IntegrityPolicy::HardFail {
                    return Err(DomainError::VaultTampered);
                }
                // WarnAndAllow: proceed and re-baseline on the next clean close.
                VaultOpenOutcome::TamperingTolerated
            }
        };

        // Mark the session in progress BEFORE touching the file, so a crash at
        // any point from here on (including mid-migration) is recognised as an
        // unclean shutdown. If this cannot be recorded, do not open.
        self.integrity_store
            .mark_session_open(self.clock.now_ms())
            .await?;
        // The keystore now owns the integrity state: legacy sidecars are moot.
        vault_integrity::remove_legacy_sidecars(&self.path);

        let db = open_encrypted_db(&self.path, dek).await?;
        let conn = connect(&db).await?;
        migrations::run(&conn, VAULT_MIGRATIONS).await?;

        let mut guard = self.state.lock().unwrap_or_else(|p| p.into_inner());
        *guard = Some(OpenVault {
            db,
            conn,
            mac_key: Zeroizing::new(mac_key.to_vec()),
        });
        Ok(outcome)
    }

    async fn close(&self) {
        // Wait for any in-flight write transaction: the baseline must describe
        // the final file, and no write may land after it.
        let _permit = self.write_lock.clone().lock_owned().await;
        // Take the open vault out and drop its DB/connection first so the
        // journal is flushed into `vault.db` before we hash the file.
        let open = {
            let mut guard = self.state.lock().unwrap_or_else(|p| p.into_inner());
            guard.take()
        };
        let Some(open) = open else {
            return;
        };
        let mac_key = open.mac_key.clone();
        drop(open);
        // Best-effort: if the baseline cannot be recorded, the session stays
        // "open" in the keystore and the next start treats it as a crash.
        if let Ok(mac) = vault_integrity::compute_mac(&self.path, mac_key.as_slice()) {
            let _ = self
                .integrity_store
                .record_clean_close(&mac, self.clock.now_ms())
                .await;
        }
    }
}

#[async_trait]
impl UnitOfWork for LibsqlVaultManager {
    async fn run<'a>(&self, work: Work<'a>) -> Result<(), DomainError> {
        self.unit_of_work(work).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::clock::SystemClock;
    use crate::infrastructure::persistence::db::open_plain_db;
    use crate::infrastructure::persistence::migrations::KEYSTORE_MIGRATIONS;

    async fn open_vault(name: &str) -> (LibsqlVaultManager, PathBuf) {
        let dir = std::env::temp_dir().join(format!("bp-vault-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let keystore = open_plain_db(&dir.join("keystore.db")).await.unwrap();
        let ks_conn = connect(&keystore).await.unwrap();
        migrations::run(&ks_conn, KEYSTORE_MIGRATIONS)
            .await
            .unwrap();
        let vault = LibsqlVaultManager::new(
            dir.join("vault.db"),
            IntegrityPolicy::HardFail,
            KeystoreIntegrityStore::new(ks_conn),
            Arc::new(SystemClock),
        );
        vault.open(&[1u8; 32], &[2u8; 32]).await.unwrap();
        vault
            .execute_write("CREATE TABLE t (v INTEGER)", ())
            .await
            .unwrap();
        (vault, dir)
    }

    async fn count(vault: &LibsqlVaultManager) -> i64 {
        let conn = vault.connection().unwrap();
        let mut rows = conn.query("SELECT COUNT(*) FROM t", ()).await.unwrap();
        rows.next().await.unwrap().unwrap().get(0).unwrap()
    }

    #[tokio::test]
    async fn a_failing_unit_of_work_rolls_back_every_write() {
        let (vault, dir) = open_vault("uow-rollback").await;
        let result: Result<(), DomainError> = vault
            .unit_of_work(async {
                vault.execute_write("INSERT INTO t VALUES (1)", ()).await?;
                vault.execute_write("INSERT INTO t VALUES (2)", ()).await?;
                Err(DomainError::Validation("boom".into()))
            })
            .await;
        assert!(result.is_err());
        assert_eq!(
            count(&vault).await,
            0,
            "nothing from the unit of work survives"
        );

        vault
            .unit_of_work(async {
                vault.execute_write("INSERT INTO t VALUES (1)", ()).await?;
                // Reads inside the unit of work see its own uncommitted writes.
                assert_eq!(count(&vault).await, 1);
                vault.execute_write("INSERT INTO t VALUES (2)", ()).await?;
                Ok(())
            })
            .await
            .unwrap();
        assert_eq!(count(&vault).await, 2);
        vault.close().await;
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn writes_after_close_are_refused() {
        let (vault, dir) = open_vault("closed").await;
        vault.close().await;
        assert!(matches!(
            vault.execute_write("INSERT INTO t VALUES (1)", ()).await,
            Err(DomainError::Unauthorized)
        ));
        let _ = std::fs::remove_dir_all(dir);
    }
}
