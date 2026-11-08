//! Keystore-backed storage of the vault's tamper-evidence state.
//!
//! The keystore is sealed at rest by the per-device key, so — unlike the legacy
//! `vault.db.hmac` / `vault.db.dirty` sidecars — this state cannot be deleted,
//! created or rewritten by someone who merely has write access to the data dir.

use libsql::{Connection, params};

use crate::domain::error::DomainError;
use crate::infrastructure::persistence::db::map_storage;

/// Integrity state of the vault as last recorded by this app.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IntegrityRecord {
    /// MAC of the vault at its last clean close (`None` before the first one).
    pub baseline_mac: Option<Vec<u8>>,
    /// A vault session was started and never cleanly closed.
    pub session_open: bool,
}

pub struct KeystoreIntegrityStore {
    conn: Connection,
}

impl KeystoreIntegrityStore {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub async fn load(&self) -> Result<IntegrityRecord, DomainError> {
        let mut rows = self
            .conn
            .query(
                "SELECT baseline_mac, session_open FROM vault_integrity WHERE id = 1",
                (),
            )
            .await
            .map_err(map_storage)?;
        let row = rows
            .next()
            .await
            .map_err(map_storage)?
            .ok_or_else(|| DomainError::Storage("vault_integrity row missing".into()))?;
        let baseline_mac: Option<Vec<u8>> = row.get(0).map_err(map_storage)?;
        let session_open: i64 = row.get(1).map_err(map_storage)?;
        Ok(IntegrityRecord {
            baseline_mac,
            session_open: session_open != 0,
        })
    }

    /// Record that a vault session is starting. Must succeed BEFORE the vault
    /// file is opened or migrated, so a crash at any later point is recognised
    /// as an unclean shutdown rather than tampering.
    pub async fn mark_session_open(&self, now_ms: i64) -> Result<(), DomainError> {
        self.conn
            .execute(
                "UPDATE vault_integrity SET session_open = 1, updated_at = ?1 WHERE id = 1",
                params![now_ms],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    /// Record a clean close: the new baseline and the end of the session are
    /// written in a single statement, hence atomically.
    pub async fn record_clean_close(&self, mac: &[u8], now_ms: i64) -> Result<(), DomainError> {
        self.conn
            .execute(
                "UPDATE vault_integrity SET baseline_mac = ?1, session_open = 0, updated_at = ?2 \
                 WHERE id = 1",
                params![mac.to_vec(), now_ms],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }
}
