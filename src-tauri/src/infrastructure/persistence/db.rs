use std::path::Path;
use std::time::Duration;

use bytes::Bytes;
use libsql::{Builder, Cipher, Connection, Database, EncryptionConfig};

use crate::domain::error::DomainError;

pub(crate) fn map_storage(e: libsql::Error) -> DomainError {
    let message = e.to_string();
    // A write pointing at a row that does not exist (unknown profile id…) is a
    // caller error the UI can explain, not an internal failure.
    if message.contains("FOREIGN KEY constraint failed") {
        return DomainError::NotFound;
    }
    DomainError::Storage(message)
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

/// How long a connection waits on a lock held by another connection of the same
/// file before failing with `SQLITE_BUSY`. Writes are serialized in-process (see
/// `LibsqlVaultManager::write`), so this only absorbs short reader/writer overlaps.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Open a connection to a built database, enabling SQLite foreign-key
/// enforcement on it. Foreign keys are off by default and the setting is
/// **per connection**, so this must run on every connection for the
/// `ON DELETE CASCADE` constraints (e.g. profile → presence) to actually fire.
pub async fn connect(db: &Database) -> Result<Connection, DomainError> {
    let conn = db.connect().map_err(map_storage)?;
    configure(&conn).await?;
    Ok(conn)
}

/// Per-connection settings shared by every connection we open.
pub(crate) async fn configure(conn: &Connection) -> Result<(), DomainError> {
    conn.busy_timeout(BUSY_TIMEOUT).map_err(map_storage)?;
    conn.execute("PRAGMA foreign_keys = ON", ())
        .await
        .map_err(map_storage)?;
    Ok(())
}
