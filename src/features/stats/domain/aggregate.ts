import {
  addMonths,
  addWeeks,
  getISOWeek,
  getISOWeekYear,
  startOfISOWeek,
  startOfMonth,
} from "date-fns";

import type {
  Presence,
  PresenceType,
} from "@/features/presence/domain/entities/presence";
import { PRESENCE_TYPES } from "@/features/presence/domain/entities/presence";

/** Time resolution for the period charts. */
export type Granularity = "month" | "week";

/** One time bucket (a calendar month or an ISO week) with its summed metrics. */
export interface PeriodBucket {
  /** Sortable identity: month = `yyyymm` style via Date.UTC; week = isoYear*100+isoWeek. */
  key: number;
  /** Local-midnight timestamp of the bucket's first day — the presentation
   * derives the localized label (month name / week number) straight from it. */
  refDay: number;
  /** Sum of non-null `co2Kg` (null days — vacation/holiday/imported — are skipped). */
  co2Kg: number;
  /** Sum of `workMinutes` over every day in the bucket. */
  workMinutes: number;
  /** Days that landed in this bucket (0 for gap-filled buckets). */
  dayCount: number;
}

/** A presence-type share of the day-count, for the breakdown pie. */
export interface BreakdownSlice {
  type: PresenceType;
  days: number;
}

/**
 * A presence's `day` is epoch-ms at UTC midnight (see `Presence.day`). Rebuild a
 * local-midnight Date carrying the SAME calendar Y/M/D so date-fns buckets it on
 * the day the user actually picked, regardless of the runtime timezone.
 */
function civilDate(dayMs: number): Date {
  const d = new Date(dayMs);
  return new Date(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate());
}

function monthKey(d: Date): number {
  return Date.UTC(d.getFullYear(), d.getMonth(), 1);
}

/** ISO week identity. Uses `getISOWeekYear` (NOT `getFullYear`): the week holding
 * Jan 1st can belong to the previous calendar year, so the ISO year is what keeps
 * end-of-December and start-of-January days in the right bucket. */
function weekKey(d: Date): number {
  return getISOWeekYear(d) * 100 + getISOWeek(d);
}

function emptyBucket(key: number, start: Date): PeriodBucket {
  return {
    key,
    refDay: start.getTime(),
    co2Kg: 0,
    workMinutes: 0,
    dayCount: 0,
  };
}

/**
 * Group every presence into month or ISO-week buckets, sorted ascending and
 * **gap-filled**: empty months/weeks between the first and last observed period
 * are inserted as zero buckets so the bars read as a continuous timeline rather
 * than silently collapsing the calendar.
 *
 * `until` (epoch-ms, typically "today") extends the timeline forward so the
 * current period stays on the chart even when the most recent days haven't been
 * encoded yet — otherwise the timeline would stop at the last filled day.
 */
export function bucketPresences(
  presences: Presence[],
  granularity: Granularity,
  until?: number,
): PeriodBucket[] {
  if (presences.length === 0) return [];

  const startOfPeriod = (d: Date) =>
    granularity === "month" ? startOfMonth(d) : startOfISOWeek(d);
  const keyOf = (d: Date) =>
    granularity === "month" ? monthKey(d) : weekKey(d);
  const step = (d: Date) =>
    granularity === "month" ? addMonths(d, 1) : addWeeks(d, 1);

  const buckets = new Map<number, PeriodBucket>();
  let min: Date | null = null;
  let max: Date | null = null;

  for (const p of presences) {
    const d = civilDate(p.day);
    const key = keyOf(d);
    let bucket = buckets.get(key);
    if (!bucket) {
      bucket = emptyBucket(key, startOfPeriod(d));
      buckets.set(key, bucket);
    }
    if (p.co2Kg != null) bucket.co2Kg += p.co2Kg;
    bucket.workMinutes += p.workMinutes;
    bucket.dayCount += 1;
    if (!min || d < min) min = d;
    if (!max || d > max) max = d;
  }

  const filled: PeriodBucket[] = [];
  let end = startOfPeriod(max!);
  // Extend the timeline up to `until` (today) so the current period is shown
  // even if the latest days carry no data.
  if (until != null) {
    const untilStart = startOfPeriod(civilDate(until));
    if (untilStart > end) end = untilStart;
  }
  // `guard` is a safety net against a malformed range looping forever.
  let cursor = startOfPeriod(min!);
  for (let guard = 0; cursor <= end && guard < 10_000; guard++) {
    const key = keyOf(cursor);
    filled.push(buckets.get(key) ?? emptyBucket(key, cursor));
    cursor = step(cursor);
  }
  return filled;
}

/**
 * The bucket key a single day (epoch-ms UTC midnight) maps to — exported so the
 * UI can filter presences to a chosen period range without re-deriving the
 * month/ISO-week logic. Keys are chronologically sortable, matching
 * `bucketPresences`, so a `start <= key <= end` test selects a contiguous range.
 */
export function periodKey(dayMs: number, granularity: Granularity): number {
  const d = civilDate(dayMs);
  return granularity === "month" ? monthKey(d) : weekKey(d);
}

/**
 * Count days per presence type, honoring the active filter set and returning
 * slices in canonical `PRESENCE_TYPES` order. Disabled types and types with no
 * days are omitted (an empty slice would just be invisible legend clutter).
 */
export function breakdownByType(
  presences: Presence[],
  enabled: ReadonlySet<PresenceType>,
): BreakdownSlice[] {
  const counts = new Map<PresenceType, number>();
  for (const p of presences) {
    if (!enabled.has(p.type)) continue;
    counts.set(p.type, (counts.get(p.type) ?? 0) + 1);
  }
  return PRESENCE_TYPES.filter((type) => (counts.get(type) ?? 0) > 0).map(
    (type) => ({ type, days: counts.get(type)! }),
  );
}
