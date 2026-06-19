use std::path::Path;

use chrono::{DateTime, NaiveDate, NaiveDateTime};
use spreadsheet_ods::style::CellStyle;
use spreadsheet_ods::{format, mm, write_ods, CellStyleRef, Sheet, ValueFormatRef, WorkBook};

use crate::domain::error::DomainError;
use crate::domain::services::spreadsheet_exporter::{
    ExportData, ExportLabels, ExportOptions, SpreadsheetExporter,
};

/// Writes an [`ExportData`] as a multi-sheet OpenDocument (`.ods`) workbook with
/// the `spreadsheet-ods` crate (MIT/Apache-2.0). Numbers are written as real
/// numbers and days as real dates, so the file opens cleanly in Excel and
/// LibreOffice without any delimiter / decimal-comma / BOM handling. All visible
/// text comes from [`ExportLabels`] (translated by the frontend), so the file
/// matches the user's language.
pub struct OdsSpreadsheetExporter;

impl OdsSpreadsheetExporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OdsSpreadsheetExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Epoch ms → naive UTC date. `day` is stored at UTC midnight, so this yields
/// the intended calendar day regardless of the reader's timezone.
fn ms_to_date(ms: i64) -> Option<NaiveDate> {
    DateTime::from_timestamp_millis(ms).map(|dt| dt.naive_utc().date())
}

/// Epoch ms → naive UTC datetime (for created/updated audit timestamps).
fn ms_to_datetime(ms: i64) -> Option<NaiveDateTime> {
    DateTime::from_timestamp_millis(ms).map(|dt| dt.naive_utc())
}

/// Styles reused across every sheet (registered once on the workbook).
struct Styles {
    header: CellStyleRef,
    date: CellStyleRef,
    datetime: CellStyleRef,
}

fn register_styles(wb: &mut WorkBook) -> Styles {
    // Value formats so date/datetime cells render as dates, not serial numbers.
    let date_fmt: ValueFormatRef =
        wb.add_datetime_format(format::create_date_dmy_format("export_date"));
    let datetime_fmt: ValueFormatRef =
        wb.add_datetime_format(format::create_datetime_format("export_datetime"));

    let mut header = CellStyle::new_empty();
    header.set_font_bold();
    let header = wb.add_cellstyle(header);

    let date = wb.add_cellstyle(CellStyle::new("export_date_cell", &date_fmt));
    let datetime = wb.add_cellstyle(CellStyle::new("export_datetime_cell", &datetime_fmt));

    Styles {
        header,
        date,
        datetime,
    }
}

impl SpreadsheetExporter for OdsSpreadsheetExporter {
    fn write_workbook(
        &self,
        data: &ExportData,
        options: &ExportOptions,
        labels: &ExportLabels,
        path: &Path,
    ) -> Result<(), DomainError> {
        let mut wb = WorkBook::new_empty();
        let styles = register_styles(&mut wb);

        wb.push_sheet(build_presences_sheet(data, options, labels, &styles));
        if options.include_tasks {
            wb.push_sheet(build_tasks_sheet(data, labels, &styles));
        }
        if options.include_trips {
            wb.push_sheet(build_trips_sheet(data, labels, &styles));
        }
        if options.include_notes {
            wb.push_sheet(build_notes_sheet(data, labels, &styles));
        }

        write_ods(&mut wb, path)
            .map_err(|e| DomainError::Export(format!("failed to write .ods file: {e}")))
    }
}

/// Présences: one row per day. Date + type are always present; the rest is gated
/// by the chosen options.
fn build_presences_sheet(
    data: &ExportData,
    options: &ExportOptions,
    labels: &ExportLabels,
    styles: &Styles,
) -> Sheet {
    let mut sheet = Sheet::new(labels.sheet_presences.as_str());

    // Header.
    let mut col = 0u32;
    sheet.set_styled_value(0, col, labels.date.as_str(), &styles.header);
    col += 1;
    sheet.set_styled_value(0, col, labels.presence_type.as_str(), &styles.header);
    col += 1;
    if options.include_carbon {
        sheet.set_styled_value(0, col, labels.co2.as_str(), &styles.header);
        col += 1;
        sheet.set_styled_value(0, col, labels.estimated.as_str(), &styles.header);
        col += 1;
    }
    if options.include_hours {
        sheet.set_styled_value(0, col, labels.hours.as_str(), &styles.header);
        col += 1;
    }
    if options.include_timestamps {
        sheet.set_styled_value(0, col, labels.created.as_str(), &styles.header);
        col += 1;
        sheet.set_styled_value(0, col, labels.updated.as_str(), &styles.header);
    }

    // Data.
    for (i, day) in data.days.iter().enumerate() {
        let row = (i + 1) as u32;
        let p = &day.presence;
        let mut col = 0u32;

        if let Some(date) = ms_to_date(p.day) {
            sheet.set_styled_value(row, col, date, &styles.date);
        }
        col += 1;
        sheet.set_value(row, col, labels.type_label(p.kind));
        col += 1;

        if options.include_carbon {
            if let Some(co2) = p.co2_kg {
                sheet.set_value(row, col, co2);
            }
            col += 1;
            sheet.set_value(row, col, labels.yes_no(p.is_estimated));
            col += 1;
        }
        if options.include_hours {
            sheet.set_value(row, col, p.work_minutes as f64 / 60.0);
            col += 1;
        }
        if options.include_timestamps {
            if let Some(dt) = ms_to_datetime(p.created_at) {
                sheet.set_styled_value(row, col, dt, &styles.datetime);
            }
            col += 1;
            if let Some(dt) = ms_to_datetime(p.updated_at) {
                sheet.set_styled_value(row, col, dt, &styles.datetime);
            }
        }
    }

    sheet.set_col_width(0, mm!(28.0));
    sheet.set_col_width(1, mm!(28.0));
    sheet.set_header_rows(0, 0);
    sheet
}

/// Tâches: one row per work entry (the day's date is repeated for context).
fn build_tasks_sheet(data: &ExportData, labels: &ExportLabels, styles: &Styles) -> Sheet {
    let mut sheet = Sheet::new(labels.sheet_tasks.as_str());
    let headers = [
        labels.date.as_str(),
        labels.title.as_str(),
        labels.description.as_str(),
        labels.minutes.as_str(),
        labels.color.as_str(),
        labels.order.as_str(),
    ];
    for (col, title) in headers.iter().enumerate() {
        sheet.set_styled_value(0, col as u32, *title, &styles.header);
    }

    let mut row = 1u32;
    for day in &data.days {
        let date = ms_to_date(day.presence.day);
        for entry in &day.entries {
            if let Some(date) = date {
                sheet.set_styled_value(row, 0, date, &styles.date);
            }
            sheet.set_value(row, 1, entry.title.as_str());
            if let Some(desc) = &entry.description {
                sheet.set_value(row, 2, desc.as_str());
            }
            sheet.set_value(row, 3, entry.minutes);
            sheet.set_value(row, 4, entry.color.as_str());
            sheet.set_value(row, 5, entry.position);
            row += 1;
        }
    }

    sheet.set_col_width(0, mm!(28.0));
    sheet.set_col_width(1, mm!(45.0));
    sheet.set_col_width(2, mm!(60.0));
    sheet.set_header_rows(0, 0);
    sheet
}

/// Trajets: one row per commute leg (the day's date is repeated for context).
fn build_trips_sheet(data: &ExportData, labels: &ExportLabels, styles: &Styles) -> Sheet {
    let mut sheet = Sheet::new(labels.sheet_trips.as_str());
    let headers = [
        labels.date.as_str(),
        labels.mode.as_str(),
        labels.distance.as_str(),
        labels.round_trip.as_str(),
        labels.occupants.as_str(),
        labels.co2.as_str(),
        labels.factor_year.as_str(),
    ];
    for (col, title) in headers.iter().enumerate() {
        sheet.set_styled_value(0, col as u32, *title, &styles.header);
    }

    let mut row = 1u32;
    for day in &data.days {
        let date = ms_to_date(day.presence.day);
        for trip in &day.trips {
            if let Some(date) = date {
                sheet.set_styled_value(row, 0, date, &styles.date);
            }
            sheet.set_value(row, 1, trip.mode_id.as_str());
            sheet.set_value(row, 2, trip.distance_km);
            sheet.set_value(row, 3, labels.yes_no(trip.round_trip));
            sheet.set_value(row, 4, trip.occupants);
            sheet.set_value(row, 5, trip.co2_kg);
            sheet.set_value(row, 6, trip.factor_year);
            row += 1;
        }
    }

    sheet.set_col_width(0, mm!(28.0));
    sheet.set_col_width(1, mm!(40.0));
    sheet.set_header_rows(0, 0);
    sheet
}

/// Notes: one row per day that has a non-blank Markdown note.
fn build_notes_sheet(data: &ExportData, labels: &ExportLabels, styles: &Styles) -> Sheet {
    let mut sheet = Sheet::new(labels.sheet_notes.as_str());
    sheet.set_styled_value(0, 0, labels.date.as_str(), &styles.header);
    sheet.set_styled_value(0, 1, labels.note.as_str(), &styles.header);

    let mut row = 1u32;
    for day in &data.days {
        let Some(note) = &day.note else { continue };
        if note.trim().is_empty() {
            continue;
        }
        if let Some(date) = ms_to_date(day.presence.day) {
            sheet.set_styled_value(row, 0, date, &styles.date);
        }
        sheet.set_value(row, 1, note.as_str());
        row += 1;
    }

    sheet.set_col_width(0, mm!(28.0));
    sheet.set_col_width(1, mm!(120.0));
    sheet.set_header_rows(0, 0);
    sheet
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::presence::{Presence, PresenceType};
    use crate::domain::entities::trip::Trip;
    use crate::domain::entities::work_entry::WorkEntry;
    use crate::domain::services::spreadsheet_exporter::DayExport;
    use calamine::{open_workbook, Data, Ods, Reader};

    fn sample_data() -> ExportData {
        // 2024-01-15 UTC midnight.
        let day = 1_705_276_800_000;
        ExportData {
            days: vec![DayExport {
                presence: Presence {
                    id: "p1".to_string(),
                    profile_id: "prof".to_string(),
                    day,
                    kind: PresenceType::Office,
                    co2_kg: Some(3.2),
                    is_estimated: false,
                    created_at: day,
                    updated_at: day,
                    work_minutes: 420,
                },
                entries: vec![WorkEntry {
                    id: "e1".to_string(),
                    presence_id: "p1".to_string(),
                    title: "Dev API".to_string(),
                    description: Some("endpoint export".to_string()),
                    minutes: 120,
                    color: "#3b82f6".to_string(),
                    position: 0,
                }],
                trips: vec![Trip {
                    id: "t1".to_string(),
                    mode_id: "train".to_string(),
                    distance_km: 40.0,
                    round_trip: true,
                    occupants: 1,
                    co2_kg: 1.1,
                    is_estimated: false,
                    factor_year: 2025,
                    position: 0,
                }],
                note: Some("Réunion projet".to_string()),
            }],
        }
    }

    fn sample_labels() -> ExportLabels {
        ExportLabels {
            sheet_presences: "Présences".to_string(),
            sheet_tasks: "Tâches".to_string(),
            sheet_trips: "Trajets".to_string(),
            sheet_notes: "Notes".to_string(),
            date: "Date".to_string(),
            yes: "Oui".to_string(),
            no: "Non".to_string(),
            co2: "CO₂ (kg)".to_string(),
            presence_type: "Type".to_string(),
            estimated: "Estimé".to_string(),
            hours: "Heures".to_string(),
            created: "Créé le".to_string(),
            updated: "Modifié le".to_string(),
            type_office: "Bureau".to_string(),
            type_remote: "Télétravail".to_string(),
            type_vacation: "Congés".to_string(),
            type_holiday: "Jour férié".to_string(),
            title: "Titre".to_string(),
            description: "Description".to_string(),
            minutes: "Minutes".to_string(),
            color: "Couleur".to_string(),
            order: "Ordre".to_string(),
            mode: "Mode".to_string(),
            distance: "Distance (km)".to_string(),
            round_trip: "Aller-retour".to_string(),
            occupants: "Occupants".to_string(),
            factor_year: "Année facteur".to_string(),
            note: "Note".to_string(),
        }
    }

    fn all_options() -> ExportOptions {
        ExportOptions {
            include_carbon: true,
            include_hours: true,
            include_tasks: true,
            include_trips: true,
            include_notes: true,
            include_timestamps: true,
        }
    }

    #[test]
    fn writes_a_readable_multi_sheet_ods() {
        let path = std::env::temp_dir().join("bp_export_writer_test.ods");
        let _ = std::fs::remove_file(&path);

        OdsSpreadsheetExporter::new()
            .write_workbook(&sample_data(), &all_options(), &sample_labels(), &path)
            .expect("export should succeed");

        let meta = std::fs::metadata(&path).expect("file should exist");
        assert!(meta.len() > 0, "file should be non-empty");

        // Round-trip: re-open with calamine and assert a few cells/sheets.
        let mut wb: Ods<_> = open_workbook(&path).expect("re-open ods");
        let names = wb.sheet_names();
        assert!(names.iter().any(|n| n == "Présences"));
        assert!(names.iter().any(|n| n == "Tâches"));
        assert!(names.iter().any(|n| n == "Trajets"));
        assert!(names.iter().any(|n| n == "Notes"));

        let range = wb
            .worksheet_range("Présences")
            .expect("Présences sheet should be readable");
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("Date".to_string()))
        );
        assert_eq!(
            range.get_value((0, 1)),
            Some(&Data::String("Type".to_string()))
        );
        assert_eq!(
            range.get_value((1, 1)),
            Some(&Data::String("Bureau".to_string()))
        );

        let _ = std::fs::remove_file(&path);
    }
}
