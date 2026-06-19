use serde::{Deserialize, Serialize};

use crate::domain::services::spreadsheet_exporter::{ExportLabels, ExportOptions};

/// Export field selection received FROM the frontend (serialized camelCase).
/// Each flag toggles a sheet and/or a set of columns in the produced file.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOptionsDto {
    pub include_carbon: bool,
    pub include_hours: bool,
    pub include_tasks: bool,
    pub include_trips: bool,
    pub include_notes: bool,
    pub include_timestamps: bool,
}

impl From<ExportOptionsDto> for ExportOptions {
    fn from(d: ExportOptionsDto) -> Self {
        ExportOptions {
            include_carbon: d.include_carbon,
            include_hours: d.include_hours,
            include_tasks: d.include_tasks,
            include_trips: d.include_trips,
            include_notes: d.include_notes,
            include_timestamps: d.include_timestamps,
        }
    }
}

/// Visible labels for the produced file, sent FROM the frontend already
/// translated to the user's language (serialized camelCase). Keeps i18n a single
/// frontend concern; the backend never hardcodes a locale.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLabelsDto {
    pub sheet_presences: String,
    pub sheet_tasks: String,
    pub sheet_trips: String,
    pub sheet_notes: String,
    pub date: String,
    pub yes: String,
    pub no: String,
    pub co2: String,
    pub presence_type: String,
    pub estimated: String,
    pub hours: String,
    pub created: String,
    pub updated: String,
    pub type_office: String,
    pub type_remote: String,
    pub type_vacation: String,
    pub type_holiday: String,
    pub title: String,
    pub description: String,
    pub minutes: String,
    pub color: String,
    pub order: String,
    pub mode: String,
    pub distance: String,
    pub round_trip: String,
    pub occupants: String,
    pub factor_year: String,
    pub note: String,
}

impl From<ExportLabelsDto> for ExportLabels {
    fn from(d: ExportLabelsDto) -> Self {
        ExportLabels {
            sheet_presences: d.sheet_presences,
            sheet_tasks: d.sheet_tasks,
            sheet_trips: d.sheet_trips,
            sheet_notes: d.sheet_notes,
            date: d.date,
            yes: d.yes,
            no: d.no,
            co2: d.co2,
            presence_type: d.presence_type,
            estimated: d.estimated,
            hours: d.hours,
            created: d.created,
            updated: d.updated,
            type_office: d.type_office,
            type_remote: d.type_remote,
            type_vacation: d.type_vacation,
            type_holiday: d.type_holiday,
            title: d.title,
            description: d.description,
            minutes: d.minutes,
            color: d.color,
            order: d.order,
            mode: d.mode,
            distance: d.distance,
            round_trip: d.round_trip,
            occupants: d.occupants,
            factor_year: d.factor_year,
            note: d.note,
        }
    }
}

/// Outcome of an export run, returned to the frontend (serialized camelCase).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSummaryDto {
    /// Number of presence days written.
    pub days: u32,
    /// Number of task (work entry) rows written.
    pub tasks: u32,
    /// Number of trip rows written.
    pub trips: u32,
    /// Absolute path the file was written to.
    pub path: String,
}
