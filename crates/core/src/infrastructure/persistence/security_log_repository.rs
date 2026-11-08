use async_trait::async_trait;
use libsql::{Connection, params};

use crate::domain::entities::security_event::{SecurityEvent, SecurityEventKind};
use crate::domain::error::DomainError;
use crate::domain::repositories::security_log::SecurityLog;
use crate::infrastructure::persistence::db::map_storage;

/// Journal size: enough for "since your last visit", small enough to never
/// become a behavioural record.
const MAX_EVENTS: i64 = 50;

/// Security journal over the keystore connection (sealed by the device key).
pub struct LibsqlSecurityLog {
    conn: Connection,
}

impl LibsqlSecurityLog {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl SecurityLog for LibsqlSecurityLog {
    async fn record(&self, kind: SecurityEventKind, at_ms: i64) -> Result<(), DomainError> {
        self.conn
            .execute(
                "INSERT INTO security_event (kind, at_ms) VALUES (?1, ?2)",
                params![kind.as_str(), at_ms],
            )
            .await
            .map_err(map_storage)?;
        self.conn
            .execute(
                "DELETE FROM security_event WHERE id NOT IN \
                 (SELECT id FROM security_event ORDER BY id DESC LIMIT ?1)",
                params![MAX_EVENTS],
            )
            .await
            .map_err(map_storage)?;
        Ok(())
    }

    async fn since(&self, after_ms: i64) -> Result<Vec<SecurityEvent>, DomainError> {
        let mut rows = self
            .conn
            .query(
                "SELECT kind, at_ms FROM security_event WHERE at_ms > ?1 ORDER BY id",
                params![after_ms],
            )
            .await
            .map_err(map_storage)?;
        let mut events = Vec::new();
        while let Some(row) = rows.next().await.map_err(map_storage)? {
            let kind: String = row.get(0).map_err(map_storage)?;
            // Unknown kinds (written by a newer version) are skipped.
            if let Some(kind) = SecurityEventKind::parse(&kind) {
                events.push(SecurityEvent {
                    kind,
                    at_ms: row.get(1).map_err(map_storage)?,
                });
            }
        }
        Ok(events)
    }

    async fn last(&self, kind: SecurityEventKind) -> Result<Option<i64>, DomainError> {
        let mut rows = self
            .conn
            .query(
                "SELECT MAX(at_ms) FROM security_event WHERE kind = ?1",
                params![kind.as_str()],
            )
            .await
            .map_err(map_storage)?;
        match rows.next().await.map_err(map_storage)? {
            Some(row) => row.get(0).map_err(map_storage),
            None => Ok(None),
        }
    }
}
