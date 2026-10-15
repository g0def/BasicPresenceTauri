use async_trait::async_trait;

use crate::domain::entities::security_event::{SecurityEvent, SecurityEventKind};
use crate::domain::error::DomainError;

/// Journal of security events (implemented over the sealed keystore).
#[async_trait]
pub trait SecurityLog: Send + Sync {
    /// Append an event; the journal keeps only the most recent entries.
    async fn record(&self, kind: SecurityEventKind, at_ms: i64) -> Result<(), DomainError>;

    /// Events strictly after `after_ms`, oldest first.
    async fn since(&self, after_ms: i64) -> Result<Vec<SecurityEvent>, DomainError>;

    /// Time of the most recent event of `kind`, if any.
    async fn last(&self, kind: SecurityEventKind) -> Result<Option<i64>, DomainError>;
}
