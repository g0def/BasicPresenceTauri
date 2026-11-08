/** Which data the user chose to include in the export. Each flag toggles a
 *  sheet and/or a set of columns in the produced .ods file (wire shape matches
 *  the backend `ExportOptionsDto` 1:1, camelCase). */
export interface ExportOptions {
  /** CO₂ column + "estimated" flag on the Présences sheet. */
  includeCarbon: boolean;
  /** Worked-hours total column on the Présences sheet. */
  includeHours: boolean;
  /** The Tâches sheet (one row per work entry). */
  includeTasks: boolean;
  /** The Trajets sheet (one row per commute leg). */
  includeTrips: boolean;
  /** The Notes sheet (one row per day that has a note). */
  includeNotes: boolean;
  /** created/updated columns on the Présences sheet. */
  includeTimestamps: boolean;
}

/** Sensible defaults: everything but the audit timestamps (rarely useful to a
 *  human reader, and noisier in the spreadsheet). */
export const DEFAULT_EXPORT_OPTIONS: ExportOptions = {
  includeCarbon: true,
  includeHours: true,
  includeTasks: true,
  includeTrips: true,
  includeNotes: true,
  includeTimestamps: false,
};

/** Every human-visible string written into the file, already translated to the
 *  user's language (matches the backend `ExportLabelsDto`, camelCase). Built
 *  from i18n by {@link buildExportLabels} so the file matches the UI language.
 *  All fields are strings except {@link ExportLabels.modeNames}. */
export interface ExportLabels {
  sheetPresences: string;
  sheetTasks: string;
  sheetTrips: string;
  sheetNotes: string;
  date: string;
  yes: string;
  no: string;
  co2: string;
  presenceType: string;
  estimated: string;
  /** Header of the one-line commute recap column on Présences. */
  tripSummary: string;
  hours: string;
  created: string;
  updated: string;
  typeOffice: string;
  typeRemote: string;
  typeVacation: string;
  typeHoliday: string;
  title: string;
  description: string;
  minutes: string;
  color: string;
  order: string;
  /** Header of the raw `mode_id` column, kept for machine reversibility. */
  mode: string;
  /** Header of the localized mode column sitting next to the raw id. */
  modeLabel: string;
  distanceOneWay: string;
  roundTrip: string;
  /** Header of the `distance × (round trip ? 2 : 1)` column. Shared by Trajets
   *  (per leg) and Présences (day total) so the two can be cross-footed. */
  distanceCounted: string;
  occupants: string;
  factorYear: string;

  // Prose fragments the backend assembles into the recap text cell.
  /** Separator between legs, e.g. `" + "` — the surrounding spaces matter. */
  tripJoin: string;
  /** Distance unit inside the recap, e.g. `"km"`. */
  unitKm: string;
  /** Round-trip marker inside the recap, e.g. `"(A/R)"`. */
  roundTripSuffix: string;
  /** One-way marker, e.g. `"(aller simple)"`. Only used when a day's legs
   *  disagree, where every leg must be qualified to stay unambiguous. */
  oneWaySuffix: string;
  /** Decimal mark for numbers rendered *inside* the recap text. Derived from
   *  Intl rather than translated — numeric cells stay real numbers and are
   *  formatted by the reader's own spreadsheet. */
  decimalSeparator: string;

  note: string;

  /** Localized transport-mode names keyed by the backend `mode_id`
   *  (`train_sncb`, `bike`, …). Unknown ids fall back to the raw id
   *  backend-side. */
  modeNames: Record<string, string>;
}

/** Outcome of an export run (matches the backend `ExportSummaryDto`). */
export interface ExportSummary {
  days: number;
  tasks: number;
  trips: number;
  /** Absolute path the file was written to. */
  path: string;
}
