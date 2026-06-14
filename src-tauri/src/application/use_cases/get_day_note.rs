use std::sync::Arc;

use crate::application::dto::day_note_dto::DayNoteDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::services::markdown::MarkdownRenderer;

/// Fetch a day's raw Markdown note and its backend-rendered, sanitized HTML.
pub struct GetDayNoteUseCase {
    presences: Arc<dyn PresenceRepository>,
    markdown: Arc<dyn MarkdownRenderer>,
}

impl GetDayNoteUseCase {
    pub fn new(
        presences: Arc<dyn PresenceRepository>,
        markdown: Arc<dyn MarkdownRenderer>,
    ) -> Self {
        Self {
            presences,
            markdown,
        }
    }

    pub async fn execute(&self, presence_id: &str) -> Result<DayNoteDto, DomainError> {
        let presence_id = presence_id.trim();
        if presence_id.is_empty() {
            return Err(DomainError::Validation(
                "presenceId is required".to_string(),
            ));
        }
        let markdown = self
            .presences
            .get_note(presence_id)
            .await?
            .unwrap_or_default();
        let html = self.markdown.render_to_safe_html(&markdown);
        Ok(DayNoteDto { markdown, html })
    }
}
