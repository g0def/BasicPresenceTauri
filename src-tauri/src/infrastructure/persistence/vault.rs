use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;
use libsql::{Connection, Database};

use crate::domain::error::DomainError;
use crate::domain::services::vault::VaultManager;
use crate::infrastructure::persistence::db::{connect, open_encrypted_db};
use crate::infrastructure::persistence::migrations::{self, VAULT_MIGRATIONS};

struct OpenVault {
    // Keep the Database alive so the connection stays valid.
    _db: Database,
    // Cloned out via `connection()` for the presence repositories.
    conn: Connection,
}

/// libSQL-backed encrypted vault. Holds an open connection only while unlocked;
/// locking drops the connection (and the in-memory key it carries).
pub struct LibsqlVaultManager {
    path: PathBuf,
    state: Mutex<Option<OpenVault>>,
}

impl LibsqlVaultManager {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
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
    async fn open(&self, dek: &[u8]) -> Result<(), DomainError> {
        let db = open_encrypted_db(&self.path, dek).await?;
        let conn = connect(&db).await?;
        migrations::run(&conn, VAULT_MIGRATIONS).await?;
        let mut guard = self.state.lock().unwrap_or_else(|p| p.into_inner());
        *guard = Some(OpenVault { _db: db, conn });
        Ok(())
    }

    fn close(&self) {
        let mut guard = self.state.lock().unwrap_or_else(|p| p.into_inner());
        *guard = None;
    }
}
