use serde::{Deserialize, Serialize};

/// An office/remote day offered for the attendance export (serialized
/// camelCase). `start/endMinutes` are the stored schedule, if any — the UI
/// flags the days where the fallback hours will apply.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceDayDto {
    /// Epoch ms at UTC midnight.
    pub day: i64,
    /// `"office"` or `"remote"`.
    pub kind: String,
    pub start_minutes: Option<i64>,
    pub end_minutes: Option<i64>,
}

/// Attendance export request received FROM the frontend.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceExportRequestDto {
    /// Written verbatim in the `employee_id` column (Odoo matches it by name).
    pub employee_name: String,
    /// Selected day keys (epoch ms at UTC midnight).
    pub days: Vec<i64>,
    pub fallback_start_minutes: i64,
    pub fallback_end_minutes: i64,
    /// Use the fallback hours for every day, ignoring the stored schedules.
    #[serde(default)]
    pub apply_to_all: bool,
}

/// Outcome of an attendance export.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceExportSummaryDto {
    /// Attendance lines written.
    pub rows: u32,
    /// Lines written with the fallback hours.
    pub fallback_rows: u32,
    /// Absolute path the file was written to.
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_deserializes_from_the_frontend_shape() {
        let json = r#"{"employeeName":"Ada","days":[0,86400000],"fallbackStartMinutes":480,"fallbackEndMinutes":1020}"#;
        let dto: AttendanceExportRequestDto = serde_json::from_str(json).unwrap();
        assert_eq!(dto.employee_name, "Ada");
        assert_eq!(dto.days, vec![0, 86_400_000]);
        assert_eq!(dto.fallback_end_minutes, 1020);
        assert!(!dto.apply_to_all, "defaults to false when omitted");
    }

    #[test]
    fn day_serializes_camel_case() {
        let dto = AttendanceDayDto {
            day: 0,
            kind: "office".into(),
            start_minutes: Some(480),
            end_minutes: None,
        };
        assert_eq!(
            serde_json::to_string(&dto).unwrap(),
            r#"{"day":0,"kind":"office","startMinutes":480,"endMinutes":null}"#
        );
    }
}
