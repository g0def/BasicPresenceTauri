use serde::Serialize;

use super::user_dto::UserDto;
use crate::domain::entities::security_event::SecurityEvent;

/// Result of a successful login (serialized camelCase: `expiresAt`, ...).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResultDto {
    pub token: String,
    /// Absolute expiry, epoch milliseconds.
    pub expires_at: i64,
    pub user: UserDto,
    /// Wrong passwords entered since the last successful login.
    pub failed_attempts_since_last_login: i64,
    /// Security events since the last successful login (lockouts, unclean
    /// shutdowns, password changes…), oldest first: the UI shows them so the
    /// owner notices activity they did not cause.
    pub security_events: Vec<SecurityEventDto>,
}

/// One security journal entry, as shown to the user.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityEventDto {
    pub kind: String,
    pub at: i64,
}

impl From<SecurityEvent> for SecurityEventDto {
    fn from(e: SecurityEvent) -> Self {
        Self {
            kind: e.kind.as_str().to_string(),
            at: e.at_ms,
        }
    }
}
