use std::sync::Arc;

use crate::application::dto::work_entry_dto::WorkDayScheduleDto;
use crate::domain::error::DomainError;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;

/// Fetch the start/end times of a presence day, if the user set them.
pub struct GetWorkScheduleUseCase {
    entries: Arc<dyn WorkEntryRepository>,
}

impl GetWorkScheduleUseCase {
    pub fn new(entries: Arc<dyn WorkEntryRepository>) -> Self {
        Self { entries }
    }

    pub async fn execute(
        &self,
        presence_id: &str,
    ) -> Result<Option<WorkDayScheduleDto>, DomainError> {
        let presence_id = presence_id.trim();
        if presence_id.is_empty() {
            return Err(DomainError::Validation(
                "presenceId is required".to_string(),
            ));
        }
        Ok(self
            .entries
            .get_schedule(presence_id)
            .await?
            .map(WorkDayScheduleDto::from))
    }
}
