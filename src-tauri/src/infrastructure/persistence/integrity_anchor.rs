use async_trait::async_trait;
use libsql::Connection;

use crate::domain::error::DomainError;
use crate::domain::services::integrity_anchor::IntegrityAnchor;
use crate::infrastructure::persistence::db::map_storage;

/// [`IntegrityAnchor`] backed by the keystore, which is itself sealed at rest by
/// the OS-keychain device key — so the counter cannot be rewritten by someone
/// holding only the data directory.
pub struct LibsqlIntegrityAnchor {
    conn: Connection,
}

impl LibsqlIntegrityAnchor {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl IntegrityAnchor for LibsqlIntegrityAnchor {
    async fn counter(&self) -> Result<u64, DomainError> {
        let mut rows = self
            .conn
            .query("SELECT counter FROM vault_baseline WHERE id = 1", ())
            .await
            .map_err(map_storage)?;
        let value: i64 = match rows.next().await.map_err(map_storage)? {
            Some(row) => row.get(0).map_err(map_storage)?,
            None => 0,
        };
        Ok(value.max(0) as u64)
    }

    async fn bump(&self) -> Result<u64, DomainError> {
        // Single statement so the read-modify-write cannot interleave.
        self.conn
            .execute(
                "INSERT INTO vault_baseline (id, counter) VALUES (1, 1) \
                 ON CONFLICT(id) DO UPDATE SET counter = counter + 1",
                (),
            )
            .await
            .map_err(map_storage)?;
        self.counter().await
    }
}
