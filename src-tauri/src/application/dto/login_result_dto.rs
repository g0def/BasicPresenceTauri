use serde::Serialize;

use super::user_dto::UserDto;

/// Result of a successful login (serialized camelCase: `expiresAt`, ...).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResultDto {
    pub token: String,
    /// Absolute expiry, epoch milliseconds.
    pub expires_at: i64,
    pub user: UserDto,
}
