import { dayKey } from "@/entities/presence/lib/day-key";

/** The presence types that count as an attendance (worked days). */
export const ATTENDANCE_KINDS = ["office", "remote"] as const;
export type AttendanceKind = (typeof ATTENDANCE_KINDS)[number];

/** An office/remote day offered for the attendance export, with its stored
 * schedule (minutes since midnight; `end` may exceed 1440), if any. */
export interface AttendanceDay {
  /** Epoch ms at UTC midnight. */
  day: number;
  kind: AttendanceKind;
  startMinutes: number | null;
  endMinutes: number | null;
}

/** What the backend needs to write the file: which days, and how to fill the
 * ones with no usable schedule. Hours are re-read from the vault. */
export interface AttendanceExportRequest {
  /** Written in the `employee_id` column (Odoo matches it by name). */
  employeeName: string;
  days: number[];
  fallbackStartMinutes: number;
  fallbackEndMinutes: number;
  /** Use the fallback hours for every day, ignoring the recorded ones (in
   * the exported file only — the stored hours are not modified). */
  applyToAll: boolean;
}

export interface AttendanceExportSummary {
  rows: number;
  fallbackRows: number;
  path: string;
}

/** Default fallback hours: 08:00 – 17:00. */
export const DEFAULT_FALLBACK = { start: 8 * 60, end: 17 * 60 } as const;

/** Day filter of the export dialog. `from`/`to` are inclusive day keys. */
export interface AttendanceFilter {
  from: number;
  to: number;
  kinds: ReadonlySet<AttendanceKind>;
}

export function filterAttendanceDays(
  days: readonly AttendanceDay[],
  { from, to, kinds }: AttendanceFilter,
): AttendanceDay[] {
  return days.filter((d) => d.day >= from && d.day <= to && kinds.has(d.kind));
}

/** Mirrors the backend rule: "apply to all", no schedule, or one without
 * duration (a fresh day with no task has end == start) → the fallback hours
 * apply. */
export function usesFallback(d: AttendanceDay, applyToAll = false): boolean {
  return (
    applyToAll ||
    d.startMinutes === null ||
    d.endMinutes === null ||
    d.endMinutes <= d.startMinutes
  );
}

/** First and last day keys of the month containing `date`, shifted by
 * `offset` months (`-1` = previous month). */
export function monthRange(
  date: Date,
  offset = 0,
): { from: number; to: number } {
  const first = new Date(date.getFullYear(), date.getMonth() + offset, 1);
  const last = new Date(date.getFullYear(), date.getMonth() + offset + 1, 0);
  return { from: dayKey(first), to: dayKey(last) };
}

/** Day key → `"yyyy-MM-dd"` (file names), in UTC like the key itself. */
export function dayKeyToIso(day: number): string {
  return new Date(day).toISOString().slice(0, 10);
}

/** Day key → `"dd/mm/yyyy"`, the format shown in the period fields (a native
 * `<input type="date">` follows the WebView's OS locale, e.g. mm/dd/yyyy). */
export function formatDayKey(day: number): string {
  const [y, m, d] = dayKeyToIso(day).split("-");
  return `${d}/${m}/${y}`;
}

/** `"dd/mm/yyyy"` (also `d/m/yyyy`, `-` or `.` separators) → day key, or
 * `null` when it is not a real calendar date. */
export function parseDayKey(text: string): number | null {
  const match = /^(\d{1,2})[/.-](\d{1,2})[/.-](\d{4})$/.exec(text.trim());
  if (!match) return null;
  const [d, m, y] = [Number(match[1]), Number(match[2]), Number(match[3])];
  const key = Date.UTC(y, m - 1, d);
  const back = new Date(key);
  return back.getUTCFullYear() === y &&
    back.getUTCMonth() === m - 1 &&
    back.getUTCDate() === d
    ? key
    : null;
}
