use std::collections::HashMap;

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
    pub trip_summary: String,
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
    pub mode_label: String,
    pub distance_one_way: String,
    pub round_trip: String,
    pub distance_counted: String,
    pub occupants: String,
    pub factor_year: String,
    pub trip_join: String,
    pub unit_km: String,
    pub round_trip_suffix: String,
    pub one_way_suffix: String,
    pub decimal_separator: String,
    pub note: String,
    /// Localized mode names keyed by `mode_id`. Note that `rename_all` only
    /// renames struct *fields* — the map's own keys stay `car_petrol`,
    /// `train_sncb`, … which is exactly what the lookup needs.
    pub mode_names: HashMap<String, String>,
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
            trip_summary: d.trip_summary,
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
            mode_label: d.mode_label,
            distance_one_way: d.distance_one_way,
            round_trip: d.round_trip,
            distance_counted: d.distance_counted,
            occupants: d.occupants,
            factor_year: d.factor_year,
            trip_join: d.trip_join,
            unit_km: d.unit_km,
            round_trip_suffix: d.round_trip_suffix,
            one_way_suffix: d.one_way_suffix,
            decimal_separator: d.decimal_separator,
            note: d.note,
            mode_names: d.mode_names,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A complete payload in the shape the frontend actually sends.
    fn wire_payload() -> serde_json::Value {
        serde_json::json!({
            "sheetPresences": "Présences",
            "sheetTasks": "Tâches",
            "sheetTrips": "Trajets",
            "sheetNotes": "Notes",
            "date": "Date",
            "yes": "Oui",
            "no": "Non",
            "co2": "CO₂ (kg)",
            "presenceType": "Type",
            "estimated": "Estimé",
            "tripSummary": "Déplacement",
            "hours": "Heures",
            "created": "Créé le",
            "updated": "Modifié le",
            "typeOffice": "Bureau",
            "typeRemote": "Télétravail",
            "typeVacation": "Congés",
            "typeHoliday": "Jour férié",
            "title": "Titre",
            "description": "Description",
            "minutes": "Minutes",
            "color": "Couleur",
            "order": "Ordre",
            "mode": "Mode (code)",
            "modeLabel": "Mode (libellé)",
            "distanceOneWay": "Distance aller (km)",
            "roundTrip": "Aller-retour",
            "distanceCounted": "Distance comptabilisée (km)",
            "occupants": "Occupants",
            "factorYear": "Année facteur",
            "tripJoin": " + ",
            "unitKm": "km",
            "roundTripSuffix": "(A/R)",
            "oneWaySuffix": "(aller simple)",
            "decimalSeparator": ",",
            "note": "Note",
            "modeNames": { "bike": "Vélo musculaire", "train_sncb": "Train Intercity (SNCB)" },
        })
    }

    #[test]
    fn export_labels_dto_deserializes_the_camel_case_wire_shape() {
        let dto: ExportLabelsDto =
            serde_json::from_value(wire_payload()).expect("the frontend payload should parse");
        let labels: ExportLabels = dto.into();

        assert_eq!(labels.distance_one_way, "Distance aller (km)");
        assert_eq!(labels.distance_counted, "Distance comptabilisée (km)");
        assert_eq!(labels.trip_summary, "Déplacement");
        assert_eq!(labels.mode_label, "Mode (libellé)");
        assert_eq!(labels.trip_join, " + ");
        assert_eq!(labels.round_trip_suffix, "(A/R)");
        assert_eq!(labels.decimal_separator, ",");
        // Map keys are NOT camel-cased by `rename_all` — the lookup relies on it.
        assert_eq!(labels.mode_name("train_sncb"), "Train Intercity (SNCB)");
    }

    #[test]
    fn export_labels_dto_rejects_a_payload_missing_mode_names() {
        let mut payload = wire_payload();
        payload
            .as_object_mut()
            .expect("payload is an object")
            .remove("modeNames");

        // Deliberately no `#[serde(default)]`: a missing key must fail loudly at
        // dev time rather than silently produce blank columns in the file.
        assert!(serde_json::from_value::<ExportLabelsDto>(payload).is_err());
    }
}
