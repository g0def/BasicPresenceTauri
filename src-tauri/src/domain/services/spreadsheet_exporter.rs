use std::collections::HashMap;
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
    /// Header of the one-line commute recap column ("Déplacement").
    pub trip_summary: String,
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
    /// Header of the raw `mode_id` column, kept for machine reversibility.
    pub mode: String,
    /// Header of the localized mode column that sits next to the raw id. Distinct
    /// from the referential's own `EmissionFactor::label`, which is French-only and
    /// frozen with its millésime.
    pub mode_label: String,
    pub distance_one_way: String,
    pub round_trip: String,
    /// Header of the `distance × (round trip ? 2 : 1)` column. Shared by Trajets
    /// (per leg) and Présences (day total) on purpose, so a reader can cross-foot
    /// the day total against the sum of its legs.
    pub distance_counted: String,
    pub occupants: String,
    pub factor_year: String,

    // Prose fragments assembled into the Présences recap text cell.
    /// Separator between legs, e.g. `" + "` (the surrounding spaces matter).
    pub trip_join: String,
    /// Distance unit inside the recap, e.g. `"km"`.
    pub unit_km: String,
    /// Round-trip marker inside the recap, e.g. `"(A/R)"`.
    pub round_trip_suffix: String,
    /// One-way marker, e.g. `"(aller simple)"`. Only used when a day's legs
    /// disagree, where every leg must be qualified to stay unambiguous.
    pub one_way_suffix: String,
    /// Decimal mark for numbers rendered *inside* the recap text. Numeric cells are
    /// written as real numbers instead, so the reader's spreadsheet formats those.
    pub decimal_separator: String,

    // Notes column.
    pub note: String,

    /// Localized transport-mode names keyed by [`Trip::mode_id`], built frontend-side
    /// from its i18n catalogue. Lookup-only — never iterated — so the map's
    /// nondeterministic ordering can never reach the produced file.
    pub mode_names: HashMap<String, String>,
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

    /// The localized name of a transport mode, falling back to the raw `mode_id`
    /// when the frontend supplied no (or a blank) translation — reachable for a
    /// mode imported from a bundle, or dropped from a later millésime.
    pub fn mode_name<'a>(&'a self, mode_id: &'a str) -> &'a str {
        match self.mode_names.get(mode_id) {
            Some(name) if !name.trim().is_empty() => name.as_str(),
            _ => mode_id,
        }
    }
}

/// Total distance actually travelled across a day's legs.
///
/// `None` when the day has no leg at all (holiday, vacation, remote day) so the
/// caller leaves the cell blank rather than writing a misleading `0`. A day whose
/// legs are all zero-distance still yields `Some(0.0)` — the option encodes "no
/// leg", never "the sum is zero".
pub fn day_counted_km(trips: &[Trip]) -> Option<f64> {
    if trips.is_empty() {
        return None;
    }
    Some(trips.iter().map(Trip::counted_km).sum())
}

/// One-line human recap of a day's commute, e.g.
/// `30 km Train Intercity (SNCB) + 4 km Vélo musculaire (A/R)`.
///
/// Distances are the stored **one-way** values; the round-trip marker is what
/// carries the doubling. Three renderings, chosen so that no two different days
/// can produce the same string:
///
/// - every leg is a round trip → one trailing marker, qualifying the journey;
/// - no leg is → no marker at all;
/// - the flags disagree (only reachable through a bundle import) → **every** leg
///   is qualified, round-trip and one-way alike. Marking only the round-trip legs
///   would render a day like `[30 km one-way, 4 km round trip]` exactly like the
///   all-round-trip day above, since both would end in a single trailing marker.
///
/// `None` for a day without legs.
pub fn summarize_trips(trips: &[Trip], labels: &ExportLabels) -> Option<String> {
    // Guard before anything else: `[].iter().all(..)` is `true`, so a day without
    // legs would otherwise render as a lone round-trip marker.
    if trips.is_empty() {
        return None;
    }
    let all_round_trip = trips.iter().all(|trip| trip.round_trip);
    let mixed = !all_round_trip && trips.iter().any(|trip| trip.round_trip);

    let legs: Vec<String> = trips
        .iter()
        .map(|trip| {
            let km = format_km(trip.distance_km, &labels.decimal_separator);
            let mut leg = [
                km.as_str(),
                labels.unit_km.trim(),
                labels.mode_name(&trip.mode_id),
            ]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
            if mixed {
                push_marker(
                    &mut leg,
                    if trip.round_trip {
                        &labels.round_trip_suffix
                    } else {
                        &labels.one_way_suffix
                    },
                );
            }
            leg
        })
        .collect();

    let mut summary = legs.join(&labels.trip_join);
    if all_round_trip {
        push_marker(&mut summary, &labels.round_trip_suffix);
    }
    Some(summary)
}

/// A distance embedded in a *text* cell: at most two decimals, no trailing `.0`
/// (`f64`'s `Display` already drops it — `Debug` would not), and the caller's
/// decimal mark. Only this prose fragment needs a fixed rendering; the numeric
/// cells next to it are written as real numbers and formatted by the reader.
fn format_km(km: f64, decimal_separator: &str) -> String {
    let rounded = (km * 100.0).round() / 100.0;
    // Checked AFTER the arithmetic, not before: `km * 100.0` overflows a finite
    // but huge input (1.8e306) to infinity, which would render as the literal
    // text "inf". The `== 0.0` arm also normalises -0.0, whose `Display` is "-0".
    if !rounded.is_finite() || rounded == 0.0 {
        return "0".to_string();
    }
    let rendered = rounded.to_string();
    match decimal_separator {
        "" | "." => rendered,
        separator => rendered.replace('.', separator),
    }
}

/// Appends ` <marker>`, ignoring a blank marker so no trailing space leaks in.
fn push_marker(target: &mut String, marker: &str) {
    let marker = marker.trim();
    if !marker.is_empty() {
        target.push(' ');
        target.push_str(marker);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn trip(mode_id: &str, distance_km: f64, round_trip: bool) -> Trip {
        Trip {
            id: format!("t-{mode_id}"),
            mode_id: mode_id.to_string(),
            distance_km,
            round_trip,
            occupants: 1,
            co2_kg: 0.0,
            is_estimated: false,
            factor_year: 2026,
            position: 0,
        }
    }

    /// French labels, matching what `buildExportLabels` sends for `fr`. The mode
    /// map deliberately carries a blank value to exercise the fallback.
    fn labels() -> ExportLabels {
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
                ("blank".to_string(), "   ".to_string()),
            ]),
        }
    }

    #[test]
    fn day_counted_km_is_none_without_trips() {
        assert_eq!(day_counted_km(&[]), None);
    }

    #[test]
    fn day_counted_km_sums_counted_distances() {
        let trips = [trip("train_sncb", 30.0, true), trip("bike", 4.0, false)];
        assert_eq!(day_counted_km(&trips), Some(64.0));
    }

    #[test]
    fn day_counted_km_is_zero_not_none_for_zero_distance_legs() {
        assert_eq!(day_counted_km(&[trip("walk", 0.0, true)]), Some(0.0));
    }

    #[test]
    fn summarize_trips_is_none_without_trips() {
        // Regression guard: `[].iter().all(..)` is `true`, so without the empty
        // guard this would render a lone "(A/R)".
        assert_eq!(summarize_trips(&[], &labels()), None);
    }

    #[test]
    fn summarize_trips_uses_one_way_distances_with_a_single_round_trip_marker() {
        let trips = [trip("train_sncb", 30.0, true), trip("bike", 4.0, true)];
        assert_eq!(
            summarize_trips(&trips, &labels()).as_deref(),
            Some("30 km Train Intercity (SNCB) + 4 km Vélo musculaire (A/R)")
        );
    }

    #[test]
    fn summarize_trips_qualifies_every_leg_when_round_trip_flags_are_mixed() {
        let trips = [trip("train_sncb", 30.0, true), trip("bike", 4.0, false)];
        assert_eq!(
            summarize_trips(&trips, &labels()).as_deref(),
            Some("30 km Train Intercity (SNCB) (A/R) + 4 km Vélo musculaire (aller simple)")
        );
    }

    #[test]
    fn summarize_trips_never_renders_two_different_days_the_same() {
        // Regression guard. Marking only the round-trip legs in the mixed case
        // made a day whose LAST leg is the round trip render byte-for-byte like
        // the all-round-trip day — same text, 68 km vs 38 km actually travelled.
        let all_round_trip = [trip("train_sncb", 30.0, true), trip("bike", 4.0, true)];
        let last_leg_only = [trip("train_sncb", 30.0, false), trip("bike", 4.0, true)];
        let none = [trip("train_sncb", 30.0, false), trip("bike", 4.0, false)];

        let rendered = [&all_round_trip, &last_leg_only, &none]
            .map(|day| summarize_trips(day, &labels()).expect("day has legs"));
        let unique: std::collections::HashSet<&String> = rendered.iter().collect();
        assert_eq!(
            unique.len(),
            3,
            "each day must render distinctly: {rendered:?}"
        );

        assert_eq!(
            rendered[1],
            "30 km Train Intercity (SNCB) (aller simple) + 4 km Vélo musculaire (A/R)"
        );
        assert_eq!(day_counted_km(&all_round_trip), Some(68.0));
        assert_eq!(day_counted_km(&last_leg_only), Some(38.0));
    }

    #[test]
    fn summarize_trips_omits_the_marker_when_no_leg_is_a_round_trip() {
        let trips = [trip("train_sncb", 30.0, false), trip("bike", 4.0, false)];
        assert_eq!(
            summarize_trips(&trips, &labels()).as_deref(),
            Some("30 km Train Intercity (SNCB) + 4 km Vélo musculaire")
        );
    }

    #[test]
    fn summarize_trips_falls_back_to_the_raw_mode_id() {
        // Absent key, then a key whose translation is blank.
        let trips = [
            trip("flying_carpet", 12.0, false),
            trip("blank", 3.0, false),
        ];
        assert_eq!(
            summarize_trips(&trips, &labels()).as_deref(),
            Some("12 km flying_carpet + 3 km blank")
        );
    }

    #[test]
    fn summarize_trips_applies_the_locale_decimal_separator() {
        let trips = [trip("bike", 4.5, false)];
        assert_eq!(
            summarize_trips(&trips, &labels()).as_deref(),
            Some("4,5 km Vélo musculaire")
        );

        let mut english = labels();
        english.decimal_separator = ".".to_string();
        assert_eq!(
            summarize_trips(&trips, &english).as_deref(),
            Some("4.5 km Vélo musculaire")
        );
    }

    #[test]
    fn summarize_trips_tolerates_blank_unit_and_marker_labels() {
        let mut bare = labels();
        bare.unit_km = String::new();
        bare.round_trip_suffix = String::new();
        assert_eq!(
            summarize_trips(&[trip("bike", 4.0, true)], &bare).as_deref(),
            Some("4 Vélo musculaire")
        );
    }

    #[test]
    fn format_km_drops_trailing_zeros_and_rounds() {
        assert_eq!(format_km(30.0, "."), "30");
        assert_eq!(format_km(4.5, "."), "4.5");
        // 12.3456 rounds up cleanly; avoid binary-half values such as 12.345.
        assert_eq!(format_km(12.3456, "."), "12.35");
        // Accumulated float error must not leak into the cell.
        assert_eq!(format_km(0.1 + 0.2, "."), "0.3");
        assert_eq!(format_km(1.0 / 3.0, "."), "0.33");
        assert_eq!(format_km(-0.0, "."), "0");
        assert_eq!(format_km(f64::NAN, "."), "0");
        assert_eq!(format_km(f64::INFINITY, "."), "0");
        // A finite input whose ×100 overflows: must not leak the text "inf".
        assert_eq!(format_km(1.8e306, "."), "0");
    }

    #[test]
    fn mode_name_falls_back_to_the_raw_id() {
        let labels = labels();
        assert_eq!(labels.mode_name("train_sncb"), "Train Intercity (SNCB)");
        assert_eq!(labels.mode_name("flying_carpet"), "flying_carpet");
        assert_eq!(labels.mode_name("blank"), "blank");
    }
}
