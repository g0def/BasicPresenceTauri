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
  createdAt: number;
  updatedAt: number;
}

/** Fields supplied when setting a presence (`id`/timestamps are backend-managed). */
export interface SetPresenceInput {
  profileId: string;
  day: number;
  type: PresenceType;
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
