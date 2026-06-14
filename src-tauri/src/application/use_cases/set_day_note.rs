use std::sync::Arc;

use crate::application::dto::day_note_dto::DayNoteDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
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
        let presence_id = presence_id.trim();
        if presence_id.is_empty() {
            return Err(DomainError::Validation(
                "presenceId is required".to_string(),
            ));
        }
        // A blank / whitespace-only note clears the column (stored as NULL).
        let trimmed = markdown.as_deref().map(str::trim).unwrap_or("");
        let stored = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };

        self.presences
            .set_note(presence_id, stored, self.clock.now_ms())
            .await?;

        let markdown = stored.unwrap_or_default().to_string();
        let html = self.markdown.render_to_safe_html(&markdown);
        Ok(DayNoteDto { markdown, html })
    }
}
