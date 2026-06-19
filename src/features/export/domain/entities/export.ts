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
 *  from i18n by {@link buildExportLabels} so the file matches the UI language. */
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
  mode: string;
  distance: string;
  roundTrip: string;
  occupants: string;
  factorYear: string;
  note: string;
}

/** Outcome of an export run (matches the backend `ExportSummaryDto`). */
export interface ExportSummary {
  days: number;
  tasks: number;
  trips: number;
  /** Absolute path the file was written to. */
  path: string;
}
