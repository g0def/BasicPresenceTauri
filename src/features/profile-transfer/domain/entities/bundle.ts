/** Profile transfer (shareable JSON bundle). Wire shapes match the backend
 *  `profile_bundle_dto` (camelCase). Distinct from the ODS export feature. */

/** Which categories to include in an export / select on import. `includeTrips`,
 *  `includeWorkHours` and `includeNotes` are children of `includeDays`. */
export interface BundleSelection {
  includeDays: boolean;
  includeTrips: boolean;
  includeWorkHours: boolean;
  includeNotes: boolean;
  includeTaskPresets: boolean;
  includeCommutes: boolean;
  includeSettings: boolean;
}

export type BundleCategoryKey = keyof BundleSelection;

/** All categories, in display order. The i18n label for each is
 *  `transfer.fields.<key>`. */
export const CATEGORY_KEYS: BundleCategoryKey[] = [
  "includeDays",
  "includeTrips",
  "includeWorkHours",
  "includeNotes",
  "includeTaskPresets",
  "includeCommutes",
  "includeSettings",
];

/** Categories that only make sense alongside the presence days. */
export const DAY_CHILD_KEYS: BundleCategoryKey[] = [
  "includeTrips",
  "includeWorkHours",
  "includeNotes",
];

/** Default: share everything. */
export const DEFAULT_SELECTION: BundleSelection = {
  includeDays: true,
  includeTrips: true,
  includeWorkHours: true,
  includeNotes: true,
  includeTaskPresets: true,
  includeCommutes: true,
  includeSettings: true,
};

/** Outcome of an export (matches `BundleExportSummaryDto`). */
export interface BundleExportSummary {
  days: number;
  trips: number;
  workEntries: number;
  notes: number;
  taskPresets: number;
  commutes: number;
  settings: boolean;
  path: string;
}

/** What `inspect_profile_bundle` reports about a file (matches
 *  `BundleManifestDto`). Drives the "intelligent" review step. */
export interface BundleManifest {
  version: number;
  compatible: boolean;
  exportedAt: number;
  app: string;
  profileFirstName: string;
  profileLastName: string;
  profileEnterprise: string;
  profilePoste?: string | null;
  days: number;
  trips: number;
  workEntries: number;
  notes: number;
  taskPresets: number;
  commutes: number;
  hasSettings: boolean;
  unknownModeIds: string[];
}

/** Per-category presence + count, derived from a manifest for the review UI. */
export interface ManifestCategory {
  key: BundleCategoryKey;
  /** Item count, or `undefined` for settings (a single row). */
  count?: number;
}

/** Which categories a file actually contains, with their counts (settings has
 *  no count). Categories absent from the file are omitted. */
export function presentCategories(m: BundleManifest): ManifestCategory[] {
  const out: ManifestCategory[] = [];
  if (m.days > 0) out.push({ key: "includeDays", count: m.days });
  if (m.trips > 0) out.push({ key: "includeTrips", count: m.trips });
  if (m.workEntries > 0)
    out.push({ key: "includeWorkHours", count: m.workEntries });
  if (m.notes > 0) out.push({ key: "includeNotes", count: m.notes });
  if (m.taskPresets > 0)
    out.push({ key: "includeTaskPresets", count: m.taskPresets });
  if (m.commutes > 0) out.push({ key: "includeCommutes", count: m.commutes });
  if (m.hasSettings) out.push({ key: "includeSettings" });
  return out;
}

/** Initial import selection: tick exactly the categories the file contains. */
export function selectionFromManifest(m: BundleManifest): BundleSelection {
  const present = new Set(presentCategories(m).map((c) => c.key));
  return {
    includeDays: present.has("includeDays"),
    includeTrips: present.has("includeTrips"),
    includeWorkHours: present.has("includeWorkHours"),
    includeNotes: present.has("includeNotes"),
    includeTaskPresets: present.has("includeTaskPresets"),
    includeCommutes: present.has("includeCommutes"),
    includeSettings: present.has("includeSettings"),
  };
}

/** Children are dropped from the payload when the days are not imported. */
export function effective(sel: BundleSelection): BundleSelection {
  if (sel.includeDays) return sel;
  return {
    ...sel,
    includeTrips: false,
    includeWorkHours: false,
    includeNotes: false,
  };
}

export type ImportTargetKind = "new" | "existing";
export type ConflictStrategy = "skip" | "replace";

/** Import destination (matches `BundleImportTargetDto`). For "new", the identity
 *  fields override the bundle's (the UI prefills them from the manifest). */
export interface BundleImportTarget {
  kind: ImportTargetKind;
  profileId?: string;
  firstName?: string;
  lastName?: string;
  enterprise?: string;
  poste?: string | null;
}

/** Per-category tallies of what an import wrote (matches `BundleImportSummaryDto`). */
export interface BundleImportSummary {
  profileId: string;
  daysImported: number;
  daysReplaced: number;
  daysSkipped: number;
  trips: number;
  workEntries: number;
  notes: number;
  taskPresets: number;
  commutes: number;
  settings: boolean;
}
