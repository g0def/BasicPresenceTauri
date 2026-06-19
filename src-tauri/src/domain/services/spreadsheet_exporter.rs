use std::path::Path;

use crate::domain::entities::presence::{Presence, PresenceType};
use crate::domain::entities::trip::Trip;
use crate::domain::entities::work_entry::WorkEntry;
use crate::domain::error::DomainError;

/// What the user chose to include in the export. Each flag gates a sheet and/or
/// a set of columns; the exact mapping lives in the exporter implementation.
#[derive(Debug, Clone, Copy)]
pub struct ExportOptions {
    /// CO₂ column + "estimated" flag on Présences (the Trajets sheet has its own
    /// toggle below).
    pub include_carbon: bool,
    /// Worked-hours total column on Présences.
    pub include_hours: bool,
    /// The per-task Tâches sheet (work entries).
    pub include_tasks: bool,
    /// The per-leg Trajets sheet (commute legs).
    pub include_trips: bool,
    /// The Notes sheet (a row per day that has a note).
    pub include_notes: bool,
    /// created_at / updated_at columns on Présences.
    pub include_timestamps: bool,
}

/// One presence day with everything attached to it, already gathered from the
/// vault. The exporter only formats this; it never touches the repositories.
#[derive(Debug, Clone)]
pub struct DayExport {
    pub presence: Presence,
    pub entries: Vec<WorkEntry>,
    pub trips: Vec<Trip>,
    /// Raw Markdown source of the day's note, when set and non-blank.
    pub note: Option<String>,
}

/// The full dataset to export for one profile (its days, ordered by day).
#[derive(Debug, Clone)]
pub struct ExportData {
    pub days: Vec<DayExport>,
}

/// Every human-visible string in the produced file (sheet names, column headers,
/// presence-type labels, yes/no). Supplied by the frontend already translated to
/// the user's language, so i18n stays a single frontend concern and the exporter
/// never hardcodes a locale.
#[derive(Debug, Clone)]
pub struct ExportLabels {
    pub sheet_presences: String,
    pub sheet_tasks: String,
    pub sheet_trips: String,
    pub sheet_notes: String,

    /// Shared across sheets.
    pub date: String,
    pub yes: String,
    pub no: String,
    pub co2: String,

    // Présences columns.
    pub presence_type: String,
    pub estimated: String,
    pub hours: String,
    pub created: String,
    pub updated: String,

    // Presence-type values.
    pub type_office: String,
    pub type_remote: String,
    pub type_vacation: String,
    pub type_holiday: String,

    // Tâches columns.
    pub title: String,
    pub description: String,
    pub minutes: String,
    pub color: String,
    pub order: String,

    // Trajets columns.
    pub mode: String,
    pub distance: String,
    pub round_trip: String,
    pub occupants: String,
    pub factor_year: String,

    // Notes column.
    pub note: String,
}

impl ExportLabels {
    /// The label for a presence type.
    pub fn type_label(&self, kind: PresenceType) -> &str {
        match kind {
            PresenceType::Office => &self.type_office,
            PresenceType::Remote => &self.type_remote,
            PresenceType::Vacation => &self.type_vacation,
            PresenceType::Holiday => &self.type_holiday,
        }
    }

    /// The label for a boolean flag.
    pub fn yes_no(&self, value: bool) -> &str {
        if value {
            &self.yes
        } else {
            &self.no
        }
    }
}

/// Port: turn an [`ExportData`] into a spreadsheet file on disk. Implemented in
/// the infrastructure layer (ODS today; an XLSX impl could slot in unchanged
/// behind this same trait).
pub trait SpreadsheetExporter: Send + Sync {
    fn write_workbook(
        &self,
        data: &ExportData,
        options: &ExportOptions,
        labels: &ExportLabels,
        path: &Path,
    ) -> Result<(), DomainError>;
}
