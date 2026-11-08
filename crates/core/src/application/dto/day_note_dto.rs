use serde::Serialize;

use crate::domain::services::markdown::MdBlock;

/// A day's free-form note. `markdown` is the raw source (fed to the editor);
/// `blocks` is its backend-rendered, inert representation for read-only display.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayNoteDto {
    pub markdown: String,
    pub blocks: Vec<MdBlock>,
}
