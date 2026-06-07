import {
  PRESENCE_TYPES,
  type ImportPresenceEntry,
  type PresenceType,
} from "@/features/presence/domain/entities/presence";

/** Why a row from the old export could not be mapped. Carries the offending
 *  value so the UI can show the user exactly what was wrong. */
export type InvalidReason =
  | { code: "missingDate" }
  | { code: "badDate"; value: string }
  | { code: "missingMode" }
  | { code: "unknownMode"; value: string };

/** A row that could not be imported, kept so the UI can list it for the user. */
export interface InvalidRow {
  /** 1-based position in the file (for display). */
  row: number;
  /** The raw `date` field as found in the file, when present. */
  date: string | null;
  reason: InvalidReason;
}

export interface ParseResult {
  entries: ImportPresenceEntry[];
  invalid: InvalidRow[];
  /** Set when the document itself is unusable (root is not a JSON array). */
  fatal?: "notArray";
}

/** Shape of one object in the old Avalonia export. */
interface RawOldEntry {
  date?: unknown;
  mode?: unknown;
  createdAt?: unknown;
  modifiedAt?: unknown;
}

const VALID_TYPES = new Set<string>(PRESENCE_TYPES);

/**
 * Calendar Y-M-D of a (local-midnight) ISO string → UTC-midnight epoch ms.
 * Only the date part is read; any time/offset is intentionally ignored so the
 * imported day lands on the intended calendar date regardless of the machine's
 * timezone.
 */
function dateToUtcMidnightMs(raw: string): number | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(raw.trim());
  if (!m) return null;
  const y = Number(m[1]);
  const mo = Number(m[2]);
  const d = Number(m[3]);
  if (mo < 1 || mo > 12 || d < 1 || d > 31) return null;
  const ms = Date.UTC(y, mo - 1, d);
  // Reject impossible dates (e.g. 2026-02-31, which JS would roll into March).
  const back = new Date(ms);
  if (
    back.getUTCFullYear() !== y ||
    back.getUTCMonth() !== mo - 1 ||
    back.getUTCDate() !== d
  ) {
    return null;
  }
  return ms;
}

/**
 * ISO instant (with offset) → epoch ms, or `undefined` when absent/null/invalid.
 * Unlike the calendar `date`, these are real timestamps so the offset matters —
 * `Date.parse` handles it.
 */
function isoToMs(raw: unknown): number | undefined {
  if (typeof raw !== "string" || raw.length === 0) return undefined;
  const ms = Date.parse(raw);
  return Number.isNaN(ms) ? undefined : ms;
}

/**
 * Map the old Avalonia export to the current model, separating valid rows from
 * invalid ones. Invalid rows never abort the import — they are returned so the
 * UI can show them to the user, who then chooses to import the valid rows.
 */
export function parseOldExport(json: unknown): ParseResult {
  if (!Array.isArray(json)) {
    return { entries: [], invalid: [], fatal: "notArray" };
  }

  const entries: ImportPresenceEntry[] = [];
  const invalid: InvalidRow[] = [];

  json.forEach((item, i) => {
    const raw = (item ?? {}) as RawOldEntry;
    const row = i + 1;
    const rawDate = typeof raw.date === "string" ? raw.date : null;

    if (rawDate === null) {
      invalid.push({ row, date: null, reason: { code: "missingDate" } });
      return;
    }
    const day = dateToUtcMidnightMs(rawDate);
    if (day === null) {
      invalid.push({
        row,
        date: rawDate,
        reason: { code: "badDate", value: rawDate },
      });
      return;
    }
    if (typeof raw.mode !== "string") {
      invalid.push({ row, date: rawDate, reason: { code: "missingMode" } });
      return;
    }
    const type = raw.mode.toLowerCase();
    if (!VALID_TYPES.has(type)) {
      invalid.push({
        row,
        date: rawDate,
        reason: { code: "unknownMode", value: raw.mode },
      });
      return;
    }

    entries.push({
      day,
      type: type as PresenceType,
      createdAt: isoToMs(raw.createdAt),
      updatedAt: isoToMs(raw.modifiedAt),
    });
  });

  return { entries, invalid };
}

/** Unique days (UTC-midnight ms) in `entries` that already have a presence. */
export function findConflicts(
  entries: ImportPresenceEntry[],
  existingDays: Set<number>,
): number[] {
  const seen = new Set<number>();
  const conflicts: number[] = [];
  for (const e of entries) {
    if (existingDays.has(e.day) && !seen.has(e.day)) {
      seen.add(e.day);
      conflicts.push(e.day);
    }
  }
  return conflicts;
}
