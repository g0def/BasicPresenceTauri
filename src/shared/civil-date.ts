/**
 * A presence's `day` is epoch-ms at **UTC midnight** (see `Presence.day`).
 *
 * Passing that number straight to `new Date(...)` and then to a date-fns helper
 * compares it in the runtime's timezone: west of UTC, 2026-03-01T00:00Z reads as
 * 2026-02-28 locally, so the day lands in the previous month. That is how a
 * monthly total silently loses (or gains) its boundary days.
 *
 * Rebuild a local-midnight Date carrying the SAME calendar Y/M/D, so every
 * comparison happens on the day the user actually picked.
 */
export function civilDate(dayMs: number): Date {
  const d = new Date(dayMs);
  return new Date(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate());
}
