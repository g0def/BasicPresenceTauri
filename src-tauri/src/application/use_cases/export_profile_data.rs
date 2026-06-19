use std::path::Path;
use std::sync::Arc;

use crate::application::dto::export_dto::{ExportLabelsDto, ExportOptionsDto, ExportSummaryDto};
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::services::spreadsheet_exporter::{DayExport, ExportData, SpreadsheetExporter};

/// Gather all of a profile's activity (presences plus their tasks, trips and
/// notes), filter it by the chosen options, and write a spreadsheet to `path`.
pub struct ExportProfileDataUseCase {
    presences: Arc<dyn PresenceRepository>,
    work_entries: Arc<dyn WorkEntryRepository>,
    exporter: Arc<dyn SpreadsheetExporter>,
}

impl ExportProfileDataUseCase {
    pub fn new(
        presences: Arc<dyn PresenceRepository>,
        work_entries: Arc<dyn WorkEntryRepository>,
        exporter: Arc<dyn SpreadsheetExporter>,
    ) -> Self {
        Self {
            presences,
            work_entries,
            exporter,
        }
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        options: ExportOptionsDto,
        labels: ExportLabelsDto,
        path: &str,
    ) -> Result<ExportSummaryDto, DomainError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(DomainError::Validation("profileId is required".to_string()));
        }
        let path = path.trim();
        if path.is_empty() {
            return Err(DomainError::Validation("path is required".to_string()));
        }
        let options = options.into();
        let labels = labels.into();

        // Gather the profile's days. Tasks/trips/notes are fetched per day: for a
        // personal tracker the volume is modest, and this reuses the existing
        // per-presence repository methods (no new SQL to maintain).
        let presences = self.presences.list_by_profile(profile_id).await?;
        let mut days = Vec::with_capacity(presences.len());
        for presence in presences {
            let entries = self.work_entries.list_by_presence(&presence.id).await?;
            let trips = self.presences.list_trips(&presence.id).await?;
            let note = self.presences.get_note(&presence.id).await?;
            days.push(DayExport {
                presence,
                entries,
                trips,
                note,
            });
        }

        let day_count = days.len() as u32;
        let task_count: u32 = days.iter().map(|d| d.entries.len() as u32).sum();
        let trip_count: u32 = days.iter().map(|d| d.trips.len() as u32).sum();

        let data = ExportData { days };
        self.exporter
            .write_workbook(&data, &options, &labels, Path::new(path))?;

        Ok(ExportSummaryDto {
            days: day_count,
            tasks: task_count,
            trips: trip_count,
            path: path.to_string(),
        })
    }
}
