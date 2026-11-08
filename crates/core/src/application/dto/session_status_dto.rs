use serde::Serialize;

/// Session validity snapshot (serialized camelCase: `remainingMs`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatusDto {
    pub valid: bool,
    pub remaining_ms: i64,
}
