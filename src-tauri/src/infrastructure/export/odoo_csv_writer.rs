//! CSV implementation of the [`AttendanceExporter`] port, in the shape Odoo's
//! attendance import (`hr.attendance`) expects:
//!
//! ```text
//! employee_id,check_in,check_out
//! Ada Lovelace,2026-10-05 08:00:00,2026-10-05 17:00:00
//! ```
//!
//! Times are naive local clock times (Odoo reads them in the importing user's
//! timezone). UTF-8 without BOM, `\n` line endings, RFC 4180 quoting.

use std::path::Path;

use crate::domain::error::DomainError;
use crate::domain::services::attendance_export::{AttendanceExporter, AttendanceRow};

const HEADER: &str = "employee_id,check_in,check_out";
const DATE_TIME: &str = "%Y-%m-%d %H:%M:%S";

pub struct OdooCsvAttendanceExporter;

impl OdooCsvAttendanceExporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OdooCsvAttendanceExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl AttendanceExporter for OdooCsvAttendanceExporter {
    fn write_attendances(
        &self,
        employee: &str,
        rows: &[AttendanceRow],
        path: &Path,
    ) -> Result<(), DomainError> {
        let csv = to_csv(employee, rows);
        crate::infrastructure::fs::write_atomic(path, csv.as_bytes())
            .map_err(|e| DomainError::Export(format!("write file: {e}")))
    }
}

/// Renders the rows as Odoo attendance CSV.
fn to_csv(employee: &str, rows: &[AttendanceRow]) -> String {
    let employee = escape(employee);
    let mut out = String::with_capacity(HEADER.len() + 1 + rows.len() * (employee.len() + 42));
    out.push_str(HEADER);
    out.push('\n');
    for row in rows {
        out.push_str(&employee);
        out.push(',');
        out.push_str(&row.check_in.format(DATE_TIME).to_string());
        out.push(',');
        out.push_str(&row.check_out.format(DATE_TIME).to_string());
        out.push('\n');
    }
    out
}

/// Quotes a field when it holds a delimiter, a quote or a line break, doubling
/// inner quotes (RFC 4180). A leading formula character (`=`, `+`, `-`, `@`)
/// is defused with an apostrophe so a spreadsheet opening the file does not
/// evaluate it.
fn escape(field: &str) -> String {
    let field = if field.starts_with(['=', '+', '-', '@']) {
        format!("'{field}")
    } else {
        field.to_string()
    };
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn row(y: i32, m: u32, d: u32, start: (u32, u32), end: (u32, u32)) -> AttendanceRow {
        let date = NaiveDate::from_ymd_opt(y, m, d).unwrap();
        AttendanceRow {
            day: 0,
            check_in: date.and_hms_opt(start.0, start.1, 0).unwrap(),
            check_out: date.and_hms_opt(end.0, end.1, 0).unwrap(),
            used_fallback: false,
        }
    }

    #[test]
    fn matches_the_odoo_import_example() {
        let rows = [
            row(2026, 10, 5, (8, 0), (17, 0)),
            row(2026, 10, 26, (8, 0), (17, 0)),
        ];
        assert_eq!(
            to_csv("Ada Lovelace", &rows),
            "employee_id,check_in,check_out\n\
             Ada Lovelace,2026-10-05 08:00:00,2026-10-05 17:00:00\n\
             Ada Lovelace,2026-10-26 08:00:00,2026-10-26 17:00:00\n"
        );
    }

    #[test]
    fn empty_export_is_just_the_header() {
        assert_eq!(to_csv("Ada", &[]), "employee_id,check_in,check_out\n");
    }

    #[test]
    fn escapes_delimiters_quotes_and_formulas() {
        assert_eq!(escape("Lovelace, Ada"), "\"Lovelace, Ada\"");
        assert_eq!(escape("Ada \"Countess\""), "\"Ada \"\"Countess\"\"\"");
        assert_eq!(escape("=1+1"), "'=1+1");
        assert_eq!(escape("Ada"), "Ada");
    }

    #[test]
    fn writes_the_file_to_disk() {
        let dir = std::env::temp_dir().join(format!("odoo-csv-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("attendance.csv");
        OdooCsvAttendanceExporter::new()
            .write_attendances("Ada", &[row(2026, 10, 5, (9, 15), (18, 0))], &path)
            .unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            text,
            "employee_id,check_in,check_out\nAda,2026-10-05 09:15:00,2026-10-05 18:00:00\n"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
