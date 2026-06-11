use std::sync::Arc;

use crate::application::dto::work_entry_dto::WorkEntryDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;

/// Fetch the work entries of a presence day, to fill the hours page.
pub struct GetWorkEntriesUseCase {
    entries: Arc<dyn WorkEntryRepository>,
}

impl GetWorkEntriesUseCase {
    pub fn new(entries: Arc<dyn WorkEntryRepository>) -> Self {
        Self { entries }
    }

    pub async fn execute(&self, presence_id: &str) -> Result<Vec<WorkEntryDto>, DomainError> {
        let presence_id = presence_id.trim();
        if presence_id.is_empty() {
            return Err(DomainError::Validation(
                "presenceId is required".to_string(),
            ));
        }
        let entries = self.entries.list_by_presence(presence_id).await?;
        Ok(entries.into_iter().map(WorkEntryDto::from).collect())
    }
}
