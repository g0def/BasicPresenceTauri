use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use libsql::{Connection, Database};
use zeroize::Zeroizing;

use crate::domain::error::DomainError;
use crate::domain::services::integrity_anchor::IntegrityAnchor;
use crate::domain::services::vault::VaultManager;
use crate::infrastructure::config::IntegrityPolicy;
use crate::infrastructure::persistence::db::{connect, open_encrypted_db};
use crate::infrastructure::persistence::migrations::{self, VAULT_MIGRATIONS};
use crate::infrastructure::persistence::vault_integrity::{self, IntegrityVerdict, SessionState};

/// How long `close()` waits for in-flight repository work before giving up on
/// writing a clean baseline. Transactions here are millisecond-scale; this bound
/// only exists so a wedged caller cannot hang application shutdown.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(2);
const DRAIN_POLL: Duration = Duration::from_millis(5);

struct OpenVault {
    // Keep the Database alive so the connection stays valid.
    _db: Database,
    // Cloned out via `connection()` for the repositories.
    conn: Connection,
    // MAC key, held only while unlocked, so `close()` can re-baseline the file.
    mac_key: Zeroizing<Vec<u8>>,
    // Session counter this baseline belongs to (anchored in the keystore).
    counter: u64,
    // Every handed-out `VaultConnection` holds a clone. `close()` waits for the
    // count to drop back to one before hashing the file.
    lease: Arc<()>,
}

/// A borrowed vault connection that keeps the vault from being baselined while
/// it is alive.
///
/// This exists because hashing the file while a transaction is still running
/// produces a baseline for a state that never lands on disk — which the next
/// launch reads as tampering. Handing out bare `Connection` clones made that
/// race reachable from ordinary use (saving a day as the session expires).
pub struct VaultConnection {
    conn: Connection,
    _lease: Arc<()>,
}

impl std::ops::Deref for VaultConnection {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        &self.conn
    }
}

/// libSQL-backed encrypted vault. Holds an open connection only while unlocked;
/// locking drops the connection (and the in-memory key it carries).
pub struct LibsqlVaultManager {
    path: PathBuf,
    integrity: IntegrityPolicy,
    anchor: Arc<dyn IntegrityAnchor>,
    state: Mutex<Option<OpenVault>>,
}

impl LibsqlVaultManager {
    pub fn new(
        path: PathBuf,
        integrity: IntegrityPolicy,
        anchor: Arc<dyn IntegrityAnchor>,
    ) -> Self {
        Self {
            path,
            integrity,
            anchor,
            state: Mutex::new(None),
        }
    }

    /// Borrow the live connection if the vault is currently unlocked. The
    /// returned guard must be held for the whole unit of work (a transaction,
    /// not just one statement).
    pub fn connection(&self) -> Option<VaultConnection> {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .map(|v| VaultConnection {
                conn: v.conn.clone(),
                _lease: v.lease.clone(),
            })
    }

    /// Snapshot the vault next to itself before a recovery re-baselines it, so a
    /// mistaken recovery is not the end of the story.
    fn back_up_before_recovery(path: &Path, counter: u64) -> Result<(), DomainError> {
        if !path.exists() {
            return Ok(());
        }
        let mut backup = path.as_os_str().to_owned();
        backup.push(format!(".recovered-{counter}.bak"));
        std::fs::copy(path, PathBuf::from(backup))
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Shared open path. `recovering` bypasses the tamper verdict after the user
    /// has explicitly acknowledged it.
    async fn open_inner(
        &self,
        dek: &[u8],
        mac_key: &[u8],
        recovering: bool,
    ) -> Result<(), DomainError> {
        // Verify at-rest integrity BEFORE opening (the open + migrations will
        // mutate the file). The session counter comes from the sealed keystore,
        // so a rolled-back sidecar no longer verifies.
        let expected_counter = self.anchor.counter().await?;
        match vault_integrity::verify(&self.path, mac_key, expected_counter)? {
            IntegrityVerdict::Clean
            | IntegrityVerdict::NoBaseline
            | IntegrityVerdict::UncleanShutdown => {}
            IntegrityVerdict::Mismatch => {
                if recovering {
                    Self::back_up_before_recovery(&self.path, expected_counter)?;
                } else if self.integrity == IntegrityPolicy::HardFail {
                    return Err(DomainError::VaultTampered);
                }
                // WarnAndAllow (or an acknowledged recovery): proceed and
                // re-baseline on the next clean close.
                // (No logging by design — the keystore/vault keep no logs.)
            }
        }

        // Claim the next session number and mark the session in progress BEFORE
        // the first byte is written. Doing this after the migrations left a
        // window where a crash — or merely a failing migration — produced a
        // mismatch with no marker, i.e. a permanent VAULT_TAMPERED on what was
        // really a power cut.
        let counter = self.anchor.bump().await?;
        vault_integrity::write_marker(&self.path, mac_key, counter, SessionState::Open)?;

        let db = open_encrypted_db(&self.path, dek).await?;
        let conn = connect(&db).await?;
        migrations::run(&conn, VAULT_MIGRATIONS).await?;

        let mut guard = self.state.lock().unwrap_or_else(|p| p.into_inner());
        *guard = Some(OpenVault {
            _db: db,
            conn,
            mac_key: Zeroizing::new(mac_key.to_vec()),
            counter,
            lease: Arc::new(()),
        });
        Ok(())
    }
}

/// Wait until every handed-out connection guard has been dropped.
/// Returns false if the deadline passes with work still in flight.
fn wait_for_drain(lease: &Arc<()>) -> bool {
    let deadline = Instant::now() + DRAIN_TIMEOUT;
    while Arc::strong_count(lease) > 1 {
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(DRAIN_POLL);
    }
    true
}

#[async_trait]
impl VaultManager for LibsqlVaultManager {
    async fn open(&self, dek: &[u8], mac_key: &[u8]) -> Result<(), DomainError> {
        self.open_inner(dek, mac_key, false).await
    }

    async fn open_recovering(&self, dek: &[u8], mac_key: &[u8]) -> Result<(), DomainError> {
        self.open_inner(dek, mac_key, true).await
    }

    fn close(&self) {
        // Take the open vault out and drop its DB/connection first so the WAL is
        // flushed into `vault.db` before we hash the file.
        let open = {
            let mut guard = self.state.lock().unwrap_or_else(|p| p.into_inner());
            guard.take()
        };
        let Some(open) = open else { return };

        let mac_key = open.mac_key.clone();
        let counter = open.counter;
        let lease = open.lease.clone();
        drop(open); // closes our handle; borrowed guards may still be alive.

        // Only baseline once the last borrowed connection is gone, otherwise the
        // digest describes a state the file has not reached yet.
        if wait_for_drain(&lease) {
            let _ = vault_integrity::write_marker(
                &self.path,
                mac_key.as_slice(),
                counter,
                SessionState::Closed,
            );
        }
        // If the drain timed out we deliberately leave the `Open` marker in
        // place: the next launch reports an unclean shutdown (tolerated) rather
        // than a mismatch against a baseline that was never true.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// In-memory anchor standing in for the keystore-backed one.
    struct MemoryAnchor(AtomicU64);

    #[async_trait]
    impl IntegrityAnchor for MemoryAnchor {
        async fn counter(&self) -> Result<u64, DomainError> {
            Ok(self.0.load(Ordering::SeqCst))
        }
        async fn bump(&self) -> Result<u64, DomainError> {
            Ok(self.0.fetch_add(1, Ordering::SeqCst) + 1)
        }
    }

    fn manager(name: &str) -> (LibsqlVaultManager, PathBuf) {
        let mut dir = std::env::temp_dir();
        dir.push(format!("bp-vault-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("vault.db");
        let anchor = Arc::new(MemoryAnchor(AtomicU64::new(0)));
        (
            LibsqlVaultManager::new(path.clone(), IntegrityPolicy::HardFail, anchor),
            dir,
        )
    }

    /// The marker must exist before the database file does, so a crash during
    /// the very first migration is read as a crash and not as tampering.
    #[test]
    fn the_session_marker_precedes_the_first_write() {
        tauri::async_runtime::block_on(async {
            let (vault, dir) = manager("marker-first");
            vault.open(&[3u8; 32], &[4u8; 32]).await.expect("open");
            let sidecar = dir.join("vault.db.hmac");
            assert!(sidecar.exists(), "sidecar written during open");
            vault.close();

            // Re-opening a cleanly closed vault verifies.
            vault.open(&[3u8; 32], &[4u8; 32]).await.expect("reopen");
            vault.close();
            let _ = std::fs::remove_dir_all(&dir);
        });
    }

    /// A borrowed connection blocks the baseline: `close()` must not hash the
    /// file while a repository still holds a handle.
    #[test]
    fn close_waits_for_borrowed_connections() {
        tauri::async_runtime::block_on(async {
            let (vault, dir) = manager("drain");
            vault.open(&[5u8; 32], &[6u8; 32]).await.expect("open");

            let borrowed = vault.connection().expect("connection");
            let lease_before = {
                let guard = vault.state.lock().unwrap();
                Arc::strong_count(&guard.as_ref().unwrap().lease)
            };
            assert_eq!(lease_before, 2, "the borrow is tracked");

            drop(borrowed);
            vault.close();
            let _ = std::fs::remove_dir_all(&dir);
        });
    }

    /// Tampering is still caught after the rework, and recovery provides a way
    /// out that keeps a copy of the suspect file.
    #[test]
    fn recovery_opens_a_tampered_vault_and_keeps_a_backup() {
        tauri::async_runtime::block_on(async {
            let (vault, dir) = manager("recover");
            let dek = [7u8; 32];
            let mac_key = [8u8; 32];
            vault.open(&dek, &mac_key).await.expect("open");
            vault.close();

            // Flip a byte in the encrypted file.
            let path = dir.join("vault.db");
            let mut bytes = std::fs::read(&path).expect("read");
            let mid = bytes.len() / 2;
            bytes[mid] ^= 0xFF;
            std::fs::write(&path, &bytes).expect("write");

            let err = vault.open(&dek, &mac_key).await.expect_err("must refuse");
            assert!(matches!(err, DomainError::VaultTampered));

            // The acknowledged recovery path gets in, and leaves a backup.
            vault
                .open_recovering(&dek, &mac_key)
                .await
                .expect("recovery opens");
            let backups: Vec<_> = std::fs::read_dir(&dir)
                .expect("read dir")
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().contains(".bak"))
                .collect();
            assert!(!backups.is_empty(), "a backup copy must be kept");

            vault.close();
            let _ = std::fs::remove_dir_all(&dir);
        });
    }
}
