use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use crate::application::dto::attendance_export_dto::{
    AttendanceExportRequestDto, AttendanceExportSummaryDto,
};
use crate::application::services::count;
use crate::domain::error::DomainError;
use crate::domain::repositories::presence_repository::PresenceRepository;
use crate::domain::repositories::work_entry_repository::WorkEntryRepository;
use crate::domain::rules;
use crate::domain::services::attendance_export::{
    AttendanceExporter, FallbackHours, build_attendance_rows,
};

/// Export the selected worked days of a profile as attendance lines (Odoo
/// CSV). The frontend only sends *which* days; the hours are re-read from the
/// vault so the file always reflects the stored data.
pub struct ExportAttendanceUseCase {
    presences: Arc<dyn PresenceRepository>,
    work_entries: Arc<dyn WorkEntryRepository>,
    exporter: Arc<dyn AttendanceExporter>,
}

impl ExportAttendanceUseCase {
    pub fn new(
        presences: Arc<dyn PresenceRepository>,
        work_entries: Arc<dyn WorkEntryRepository>,
        exporter: Arc<dyn AttendanceExporter>,
    ) -> Self {
        Self {
            presences,
            work_entries,
            exporter,
        }
    }

    /// Validate the request before any dialog is shown.
    pub fn validate(request: &AttendanceExportRequestDto) -> Result<(), DomainError> {
        rules::employee_name(&request.employee_name)?;
        rules::attendance_fallback(request.fallback_start_minutes, request.fallback_end_minutes)?;
        rules::attendance_days(&request.days)
    }

    pub async fn execute(
        &self,
        profile_id: &str,
        request: AttendanceExportRequestDto,
        path: &Path,
    ) -> Result<AttendanceExportSummaryDto, DomainError> {
        let profile_id = rules::required_id("profileId", profile_id)?;
        rules::attendance_days(&request.days)?;
        let employee = rules::employee_name(&request.employee_name)?;
        let (start_minutes, end_minutes) = rules::attendance_fallback(
            request.fallback_start_minutes,
            request.fallback_end_minutes,
        )?;
        let selected: BTreeSet<i64> = request.days.into_iter().collect();

        let presences = self.presences.list_by_profile(profile_id).await?;
        let schedules = self
            .work_entries
            .list_schedules_by_profile(profile_id)
            .await?;
        let rows = build_attendance_rows(
            &presences,
            &schedules,
            &selected,
            FallbackHours {
                start_minutes,
                end_minutes,
                apply_to_all: request.apply_to_all,
            },
        )?;
        self.exporter.write_attendances(&employee, &rows, path)?;

        Ok(AttendanceExportSummaryDto {
            rows: count(rows.len()),
            fallback_rows: count(rows.iter().filter(|r| r.used_fallback).count()),
            path: path.display().to_string(),
        })
    }
}
