use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;
use libsql::{Connection, Database};
use zeroize::Zeroizing;

use crate::domain::error::DomainError;
use crate::domain::services::vault::VaultManager;
use crate::infrastructure::config::IntegrityPolicy;
use crate::infrastructure::persistence::db::{connect, open_encrypted_db};
use crate::infrastructure::persistence::migrations::{self, VAULT_MIGRATIONS};
use crate::infrastructure::persistence::vault_integrity::{self, IntegrityVerdict};

struct OpenVault {
    // Keep the Database alive so the connection stays valid.
    _db: Database,
    // Cloned out via `connection()` for the presence repositories.
    conn: Connection,
    // MAC key, held only while unlocked, so `close()` can re-baseline the file.
    mac_key: Zeroizing<Vec<u8>>,
}

/// libSQL-backed encrypted vault. Holds an open connection only while unlocked;
/// locking drops the connection (and the in-memory key it carries).
pub struct LibsqlVaultManager {
    path: PathBuf,
    integrity: IntegrityPolicy,
    state: Mutex<Option<OpenVault>>,
}

impl LibsqlVaultManager {
    pub fn new(path: PathBuf, integrity: IntegrityPolicy) -> Self {
        Self {
            path,
            integrity,
            state: Mutex::new(None),
        }
    }

    /// Clone the live connection if the vault is currently unlocked
    /// (used by the presence repositories).
    pub fn connection(&self) -> Option<Connection> {
        self.state
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .as_ref()
            .map(|v| v.conn.clone())
    }
}

#[async_trait]
impl VaultManager for LibsqlVaultManager {
    async fn open(&self, dek: &[u8], mac_key: &[u8]) -> Result<(), DomainError> {
        // Verify at-rest integrity BEFORE opening (the open + migrations will
        // mutate the file). A mismatch after a clean shutdown is treated per the
        // configured policy; an unclean shutdown / missing baseline is expected.
        match vault_integrity::verify(&self.path, mac_key)? {
            IntegrityVerdict::Clean
            | IntegrityVerdict::NoBaseline
            | IntegrityVerdict::UncleanShutdown => {}
            IntegrityVerdict::Mismatch => {
                if self.integrity == IntegrityPolicy::HardFail {
                    return Err(DomainError::VaultTampered);
                }
                // WarnAndAllow: proceed and re-baseline on the next clean close.
                // (No logging by design — the keystore/vault keep no logs.)
            }
        }

        let db = open_encrypted_db(&self.path, dek).await?;
        let conn = connect(&db).await?;
        migrations::run(&conn, VAULT_MIGRATIONS).await?;

        let mut guard = self.state.lock().unwrap_or_else(|p| p.into_inner());
        *guard = Some(OpenVault {
            _db: db,
            conn,
            mac_key: Zeroizing::new(mac_key.to_vec()),
        });

        // Mark the session in progress so a crash before close is recognised.
        let _ = vault_integrity::mark_dirty(&self.path);
        Ok(())
    }

    fn close(&self) {
        // Take the open vault out and drop its DB/connection first so the WAL is
        // flushed into `vault.db` before we hash the file.
        let open = {
            let mut guard = self.state.lock().unwrap_or_else(|p| p.into_inner());
            guard.take()
        };
        if let Some(open) = open {
            let mac_key = open.mac_key.clone();
            drop(open); // closes the DB, checkpointing the WAL into the main file.
                        // Best-effort: refresh the baseline and clear the dirty marker.
            let _ = vault_integrity::write_baseline(&self.path, mac_key.as_slice());
        }
    }
}
