/** The four presence types, in display order. */
export const PRESENCE_TYPES = [
  "office",
  "remote",
  "vacation",
  "holiday",
] as const;

export type PresenceType = (typeof PRESENCE_TYPES)[number];

export interface Presence {
  id: string;
  profileId: string;
  /** Epoch ms at UTC midnight of the day concerned. */
  day: number;
  type: PresenceType;
  /** Day commute footprint in kgCO2e. `null` when not computed (imported days,
   * or days never encoded with the CO2 feature); `0` is a real computed value
   * (e.g. télétravail with no trip). */
  co2Kg: number | null;
  /** True if any of the day's trips fell back to an estimated factor. */
  isEstimated: boolean;
  createdAt: number;
  updatedAt: number;
  /** Sum of minutes of work entries for this day. */
  workMinutes: number;
}

/** One commute leg attached to a presence day. Structurally identical to the
 * commute feature's `TripInput`; declared here so presence stays decoupled. */
export interface PresenceTrip {
  modeId: string;
  distanceKm: number;
  roundTrip: boolean;
  occupants: number;
}

/** Fields supplied when setting a presence (`id`/timestamps are backend-managed).
 * `trips` carries the commute legs for office/remote days (cleared otherwise). */
export interface SetPresenceInput {
  profileId: string;
  day: number;
  type: PresenceType;
  trips?: PresenceTrip[];
}

/**
 * A single presence row to import. `id` is backend-managed; the timestamps are
 * carried over from the old export when present (the backend falls back to
 * "now" otherwise).
 */
export interface ImportPresenceEntry {
  day: number;
  type: PresenceType;
  createdAt?: number;
  updatedAt?: number;
}

/** Outcome of an import run (`imported + replaced + skipped === total`). */
export interface ImportSummary {
  imported: number;
  skipped: number;
  replaced: number;
  total: number;
}

/** Arguments for a bulk import into a profile. */
export interface ImportPresencesInput {
  profileId: string;
  entries: ImportPresenceEntry[];
  /** `false` keeps existing days, `true` overwrites them. */
  replaceExisting: boolean;
}
