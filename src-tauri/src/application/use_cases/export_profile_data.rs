use std::path::Path;
use std::sync::Arc;

use crate::application::dto::export_dto::{ExportLabelsDto, ExportOptionsDto, ExportSummaryDto};
use crate::application::services::count;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::rules;
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
        path: &Path,
    ) -> Result<ExportSummaryDto, DomainError> {
        let profile_id = rules::required_id("profileId", profile_id)?;
        labels.validate()?;
        let options = options.into();
        let labels = labels.into();

        // Gather the profile's days: one query per kind of data, joined in
        // memory (no per-day N+1).
        let presences = self.presences.list_by_profile(profile_id).await?;
        let mut trips = self.presences.list_trips_by_profile(profile_id).await?;
        let mut notes = self.presences.list_notes_by_profile(profile_id).await?;
        let mut entries = self.work_entries.list_by_profile(profile_id).await?;
        let days: Vec<DayExport> = presences
            .into_iter()
            .map(|presence| DayExport {
                entries: entries.remove(&presence.id).unwrap_or_default(),
                trips: trips.remove(&presence.id).unwrap_or_default(),
                note: notes.remove(&presence.id),
                presence,
            })
            .collect();

        let day_count = count(days.len());
        let task_count: u32 = days.iter().map(|d| count(d.entries.len())).sum();
        let trip_count: u32 = days.iter().map(|d| count(d.trips.len())).sum();

        let data = ExportData { days };
        self.exporter
            .write_workbook(&data, &options, &labels, path)?;

        Ok(ExportSummaryDto {
            days: day_count,
            tasks: task_count,
            trips: trip_count,
            path: path.display().to_string(),
        })
    }
}
