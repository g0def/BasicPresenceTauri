use serde::{Deserialize, Serialize};

/// One presence row received FROM the frontend importer. `day`/`type` are
/// already normalized by the frontend (UTC-midnight epoch ms + lowercase type);
/// the optional timestamps are carried over from the old export when present
/// (falling back to "now" server-side otherwise).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPresenceEntryDto {
    pub day: i64,
    #[serde(rename = "type")]
    pub kind: String,
    /// Epoch ms; `None` (absent/null) falls back to `now`.
    pub created_at: Option<i64>,
    /// Epoch ms; `None` (absent/null) falls back to `created_at` (then `now`).
    pub updated_at: Option<i64>,
}

/// Outcome of an import run, returned to the frontend (serialized camelCase).
/// `imported + replaced + skipped == total` always holds.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummaryDto {
    /// Rows that did not previously exist and were inserted.
    pub imported: u32,
    /// Existing rows left untouched (skip strategy / `replace_existing == false`).
    pub skipped: u32,
    /// Existing rows overwritten (replace strategy only).
    pub replaced: u32,
    /// Total rows received in the request.
    pub total: u32,
}
