use std::sync::Arc;

use crate::application::dto::day_note_dto::DayNoteDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::rules;
use crate::domain::services::clock::Clock;
use crate::domain::services::markdown::MarkdownRenderer;

/// Persist a day's raw Markdown note (a blank note clears it) and return it
/// alongside its freshly rendered, sanitized HTML.
pub struct SetDayNoteUseCase {
    presences: Arc<dyn PresenceRepository>,
    markdown: Arc<dyn MarkdownRenderer>,
    clock: Arc<dyn Clock>,
}

impl SetDayNoteUseCase {
    pub fn new(
        presences: Arc<dyn PresenceRepository>,
        markdown: Arc<dyn MarkdownRenderer>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            presences,
            markdown,
            clock,
        }
    }

    pub async fn execute(
        &self,
        presence_id: &str,
        markdown: Option<String>,
    ) -> Result<DayNoteDto, DomainError> {
        let presence_id = rules::required_id("presenceId", presence_id)?;
        // A blank / whitespace-only note clears the column (stored as NULL);
        // otherwise its size is bounded (it is re-rendered on every read).
        let stored = rules::note(markdown.as_deref())?;

        self.presences
            .set_note(presence_id, stored.as_deref(), self.clock.now_ms())
            .await?;

        let markdown = stored.unwrap_or_default();
        let html = self.markdown.render_to_safe_html(&markdown);
        Ok(DayNoteDto { markdown, html })
    }
}
