use std::path::Path;

use bytes::Bytes;
use libsql::{Builder, Cipher, Connection, Database, EncryptionConfig};

use crate::domain::error::DomainError;

pub(crate) fn map_storage(e: libsql::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

/// Commit the current transaction, rolling back if the commit itself fails.
///
/// Every repository shares one connection to the open vault. A `COMMIT` that
/// errors leaves the transaction *open* on that shared connection, so the next
/// write — from any repository — silently joins a doomed transaction and is
/// lost when it eventually rolls back. Propagating the error without cleaning up
/// turns one failed save into an unbounded stretch of writes that report success
/// and never land.
pub(crate) async fn commit_or_rollback(conn: &Connection) -> Result<(), DomainError> {
    match conn.execute("COMMIT", ()).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            Err(map_storage(e))
        }
    }
}

/// Open a plaintext local libSQL database (used for the keystore).
pub async fn open_plain_db(path: &Path) -> Result<Database, DomainError> {
    Builder::new_local(path).build().await.map_err(map_storage)
}

/// Open an encrypted local libSQL database (the vault) with a 32-byte key.
///
/// Uses libSQL's AES-256-CBC at-rest encryption; the DEK is supplied as the raw
/// 32-byte key (never derived by libSQL itself).
pub async fn open_encrypted_db(path: &Path, dek: &[u8]) -> Result<Database, DomainError> {
    let config = EncryptionConfig::new(Cipher::Aes256Cbc, Bytes::copy_from_slice(dek));
    Builder::new_local(path)
        .encryption_config(config)
        .build()
        .await
        .map_err(map_storage)
}

/// Open a connection to a built database, enabling SQLite foreign-key
/// enforcement on it. Foreign keys are off by default and the setting is
/// **per connection**, so this must run on every connection for the
/// `ON DELETE CASCADE` constraints (e.g. profile → presence) to actually fire.
pub async fn connect(db: &Database) -> Result<Connection, DomainError> {
    let conn = db.connect().map_err(map_storage)?;
    conn.execute("PRAGMA foreign_keys = ON", ())
        .await
        .map_err(map_storage)?;
    Ok(conn)
}
