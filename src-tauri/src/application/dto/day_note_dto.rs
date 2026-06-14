use serde::Serialize;

/// A day's free-form note returned to the frontend (serialized camelCase).
/// `markdown` is the raw source (fed to the WYSIWYG editor); `html` is the
/// backend-rendered, sanitized representation used for read-only display.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayNoteDto {
    pub markdown: String,
    pub html: String,
}
