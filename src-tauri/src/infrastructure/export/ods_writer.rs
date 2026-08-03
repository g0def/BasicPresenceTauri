use std::path::Path;

use chrono::{DateTime, NaiveDate, NaiveDateTime};
use spreadsheet_ods::style::CellStyle;
use spreadsheet_ods::{format, mm, write_ods, CellStyleRef, Sheet, ValueFormatRef, WorkBook};

use crate::domain::error::DomainError;
use crate::domain::services::spreadsheet_exporter::{
    day_counted_km, summarize_trips, ExportData, ExportLabels, ExportOptions, SpreadsheetExporter,
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

    // Header. Every block below must leave `col` on the next free column, whether
    // or not it wrote anything — an early `col += 1` shifts every later column.
    let mut col = 0u32;
    // Index of the commute recap column, captured here because it depends on which
    // earlier blocks are enabled; used to widen the two columns at the end.
    let mut summary_col: Option<u32> = None;
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
    if options.include_trips {
        summary_col = Some(col);
        sheet.set_styled_value(0, col, labels.trip_summary.as_str(), &styles.header);
        col += 1;
        sheet.set_styled_value(0, col, labels.distance_counted.as_str(), &styles.header);
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
        if options.include_trips {
            // Left blank (not `0`) for a day without legs — a holiday or a remote
            // day still carries building-energy CO₂ but travelled no kilometre.
            if let Some(summary) = summarize_trips(&day.trips, labels) {
                sheet.set_value(row, col, summary.as_str());
            }
            col += 1;
            if let Some(km) = day_counted_km(&day.trips) {
                sheet.set_value(row, col, km);
            }
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
    if let Some(c) = summary_col {
        sheet.set_col_width(c, mm!(90.0));
        sheet.set_col_width(c + 1, mm!(42.0));
    }
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
    // Interim layout: lot 3 (export auditable) will slot the factor value, its
    // unit and its dated source in around the activity data.
    let headers = [
        labels.date.as_str(),             // 0
        labels.mode.as_str(),             // 1 raw mode_id, machine-reversible
        labels.mode_label.as_str(),       // 2 localized
        labels.distance_one_way.as_str(), // 3 as stored
        labels.round_trip.as_str(),       // 4
        labels.distance_counted.as_str(), // 5 = 3 × (4 ? 2 : 1)
        labels.occupants.as_str(),        // 6
        labels.co2.as_str(),              // 7
        labels.factor_year.as_str(),      // 8
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
            sheet.set_value(row, 2, labels.mode_name(&trip.mode_id));
            sheet.set_value(row, 3, trip.distance_km);
            sheet.set_value(row, 4, labels.yes_no(trip.round_trip));
            sheet.set_value(row, 5, trip.counted_km());
            sheet.set_value(row, 6, trip.occupants);
            sheet.set_value(row, 7, trip.co2_kg);
            sheet.set_value(row, 8, trip.factor_year);
            row += 1;
        }
    }

    sheet.set_col_width(0, mm!(28.0));
    sheet.set_col_width(1, mm!(34.0));
    sheet.set_col_width(2, mm!(48.0));
    sheet.set_col_width(3, mm!(30.0));
    sheet.set_col_width(5, mm!(42.0));
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
    use std::collections::HashMap;
    use std::path::PathBuf;

    /// 2024-01-15 UTC midnight.
    const DAY: i64 = 1_705_276_800_000;

    fn trip(mode_id: &str, distance_km: f64, round_trip: bool, co2_kg: f64, position: i64) -> Trip {
        Trip {
            id: format!("t{position}"),
            mode_id: mode_id.to_string(),
            distance_km,
            round_trip,
            occupants: 1,
            co2_kg,
            is_estimated: false,
            factor_year: 2025,
            position,
        }
    }

    fn presence(kind: PresenceType, co2_kg: Option<f64>, work_minutes: i64) -> Presence {
        Presence {
            id: "p1".to_string(),
            profile_id: "prof".to_string(),
            day: DAY,
            kind,
            co2_kg,
            is_estimated: false,
            created_at: DAY,
            updated_at: DAY,
            work_minutes,
        }
    }

    /// One office day: 30 km SNCB + 4 km bike, both round trips (counted 68 km).
    fn sample_data() -> ExportData {
        ExportData {
            days: vec![DayExport {
                presence: presence(PresenceType::Office, Some(3.2), 420),
                entries: vec![WorkEntry {
                    id: "e1".to_string(),
                    presence_id: "p1".to_string(),
                    title: "Dev API".to_string(),
                    description: Some("endpoint export".to_string()),
                    minutes: 120,
                    color: "#3b82f6".to_string(),
                    position: 0,
                }],
                // Distinct footprints so the CO₂ column (which this change moved
                // from index 5 to 7) is pinned to a value, not to a blank cell.
                trips: vec![
                    trip("train_sncb", 30.0, true, 1.26, 0),
                    trip("bike", 4.0, true, 0.0, 1),
                ],
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
            trip_summary: "Déplacement".to_string(),
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
            mode: "Mode (code)".to_string(),
            mode_label: "Mode (libellé)".to_string(),
            distance_one_way: "Distance aller (km)".to_string(),
            round_trip: "Aller-retour".to_string(),
            distance_counted: "Distance comptabilisée (km)".to_string(),
            occupants: "Occupants".to_string(),
            factor_year: "Année facteur".to_string(),
            trip_join: " + ".to_string(),
            unit_km: "km".to_string(),
            round_trip_suffix: "(A/R)".to_string(),
            one_way_suffix: "(aller simple)".to_string(),
            decimal_separator: ",".to_string(),
            note: "Note".to_string(),
            mode_names: HashMap::from([
                (
                    "train_sncb".to_string(),
                    "Train Intercity (SNCB)".to_string(),
                ),
                ("bike".to_string(), "Vélo musculaire".to_string()),
            ]),
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

    /// Writes a workbook to its own file: these tests run in parallel, so a shared
    /// path would have them racing on the same bytes.
    fn write_to_temp(file_name: &str, data: &ExportData, options: &ExportOptions) -> PathBuf {
        let path = std::env::temp_dir().join(file_name);
        let _ = std::fs::remove_file(&path);
        OdsSpreadsheetExporter::new()
            .write_workbook(data, options, &sample_labels(), &path)
            .expect("export should succeed");
        path
    }

    fn text(value: Option<&Data>) -> String {
        match value {
            Some(Data::String(s)) => s.clone(),
            other => panic!("expected a string cell, got {other:?}"),
        }
    }

    /// ODS numeric cells may come back as either variant depending on the value.
    fn num(value: Option<&Data>) -> f64 {
        match value {
            Some(Data::Float(f)) => *f,
            Some(Data::Int(i)) => *i as f64,
            other => panic!("expected a numeric cell, got {other:?}"),
        }
    }

    fn headers(range: &calamine::Range<Data>, count: u32) -> Vec<String> {
        (0..count).map(|c| text(range.get_value((0, c)))).collect()
    }

    #[test]
    fn writes_a_readable_multi_sheet_ods() {
        let path = write_to_temp("bp_export_writer_test.ods", &sample_data(), &all_options());

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

    #[test]
    fn presences_sheet_carries_the_trip_summary_and_counted_distance() {
        let path = write_to_temp(
            "bp_export_presences_trips.ods",
            &sample_data(),
            &all_options(),
        );
        let mut wb: Ods<_> = open_workbook(&path).expect("re-open ods");
        let range = wb.worksheet_range("Présences").expect("Présences");

        assert_eq!(
            headers(&range, 9),
            vec![
                "Date",
                "Type",
                "CO₂ (kg)",
                "Estimé",
                "Déplacement",
                "Distance comptabilisée (km)",
                "Heures",
                "Créé le",
                "Modifié le",
            ]
        );
        assert_eq!(
            text(range.get_value((1, 4))),
            "30 km Train Intercity (SNCB) + 4 km Vélo musculaire (A/R)"
        );
        assert_eq!(num(range.get_value((1, 5))), 68.0);
        // A real number, not a locale-formatted string: the reader must be able to
        // sum the column.
        assert!(matches!(range.get_value((1, 5)), Some(Data::Float(_))));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn presences_sheet_omits_the_trip_columns_when_trips_are_excluded() {
        let options = ExportOptions {
            include_trips: false,
            ..all_options()
        };
        let path = write_to_temp("bp_export_presences_no_trips.ods", &sample_data(), &options);
        let mut wb: Ods<_> = open_workbook(&path).expect("re-open ods");
        assert!(!wb.sheet_names().iter().any(|n| n == "Trajets"));

        let range = wb.worksheet_range("Présences").expect("Présences");
        // The running column counter must not have advanced: hours slides back to 4.
        assert_eq!(
            headers(&range, 7),
            vec![
                "Date",
                "Type",
                "CO₂ (kg)",
                "Estimé",
                "Heures",
                "Créé le",
                "Modifié le",
            ]
        );
        assert_eq!(num(range.get_value((1, 4))), 7.0);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn presences_sheet_places_the_trip_columns_right_when_carbon_is_excluded() {
        // The one reachable layout where the recap does NOT land at index 4: the
        // two carbon columns are gone, so it slides to 2/3. Every flag is
        // independently toggleable in the dialog, so this is a real user config.
        let options = ExportOptions {
            include_carbon: false,
            ..all_options()
        };
        let path = write_to_temp(
            "bp_export_presences_no_carbon.ods",
            &sample_data(),
            &options,
        );
        let mut wb: Ods<_> = open_workbook(&path).expect("re-open ods");
        let range = wb.worksheet_range("Présences").expect("Présences");

        assert_eq!(
            headers(&range, 7),
            vec![
                "Date",
                "Type",
                "Déplacement",
                "Distance comptabilisée (km)",
                "Heures",
                "Créé le",
                "Modifié le",
            ]
        );
        assert_eq!(
            text(range.get_value((1, 2))),
            "30 km Train Intercity (SNCB) + 4 km Vélo musculaire (A/R)"
        );
        assert_eq!(num(range.get_value((1, 3))), 68.0);
        assert_eq!(num(range.get_value((1, 4))), 7.0);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn presences_trip_cells_are_blank_for_a_day_without_trips() {
        let data = ExportData {
            days: vec![DayExport {
                presence: presence(PresenceType::Vacation, None, 0),
                entries: Vec::new(),
                trips: Vec::new(),
                note: None,
            }],
        };
        let path = write_to_temp("bp_export_presences_no_legs.ods", &data, &all_options());
        let mut wb: Ods<_> = open_workbook(&path).expect("re-open ods");
        let range = wb.worksheet_range("Présences").expect("Présences");

        // Blank, never `0` — a day with no leg travelled no kilometre, which is not
        // the same statement as "travelled zero kilometres".
        assert!(matches!(range.get_value((1, 4)), None | Some(Data::Empty)));
        assert!(matches!(range.get_value((1, 5)), None | Some(Data::Empty)));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn trips_sheet_keeps_the_raw_mode_and_adds_the_localized_label() {
        let path = write_to_temp("bp_export_trips_sheet.ods", &sample_data(), &all_options());
        let mut wb: Ods<_> = open_workbook(&path).expect("re-open ods");
        let range = wb.worksheet_range("Trajets").expect("Trajets");

        assert_eq!(
            headers(&range, 9),
            vec![
                "Date",
                "Mode (code)",
                "Mode (libellé)",
                "Distance aller (km)",
                "Aller-retour",
                "Distance comptabilisée (km)",
                "Occupants",
                "CO₂ (kg)",
                "Année facteur",
            ]
        );
        assert_eq!(text(range.get_value((1, 1))), "train_sncb");
        assert_eq!(text(range.get_value((1, 2))), "Train Intercity (SNCB)");
        assert_eq!(num(range.get_value((1, 3))), 30.0);
        assert_eq!(text(range.get_value((1, 4))), "Oui");
        assert_eq!(num(range.get_value((1, 5))), 60.0);
        assert_eq!(num(range.get_value((1, 6))), 1.0);
        assert_eq!(num(range.get_value((1, 7))), 1.26);
        assert_eq!(num(range.get_value((1, 8))), 2025.0);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn trips_sheet_falls_back_to_the_raw_mode_id_for_an_unknown_mode() {
        let mut data = sample_data();
        data.days[0].trips = vec![trip("flying_carpet", 12.0, false, 2.77, 0)];
        let path = write_to_temp("bp_export_trips_unknown_mode.ods", &data, &all_options());
        let mut wb: Ods<_> = open_workbook(&path).expect("re-open ods");
        let range = wb.worksheet_range("Trajets").expect("Trajets");

        assert_eq!(text(range.get_value((1, 1))), "flying_carpet");
        assert_eq!(text(range.get_value((1, 2))), "flying_carpet");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn presences_counted_distance_equals_the_sum_of_the_trips_sheet() {
        let path = write_to_temp(
            "bp_export_distance_crossfoot.ods",
            &sample_data(),
            &all_options(),
        );
        let mut wb: Ods<_> = open_workbook(&path).expect("re-open ods");

        let trips = wb.worksheet_range("Trajets").expect("Trajets");
        let legs: f64 = (1..trips.height() as u32)
            .map(|row| num(trips.get_value((row, 5))))
            .sum();

        let presences = wb.worksheet_range("Présences").expect("Présences");
        assert_eq!(num(presences.get_value((1, 5))), legs);

        let _ = std::fs::remove_file(&path);
    }
}
