/**
 * Canonical key for a calendar day: epoch ms at UTC midnight of its Y/M/D.
 * Matches the `day` value the Rust backend stores, so the frontend can look up
 * a presence by the day a user clicked regardless of the local timezone.
 */
export function dayKey(date: Date): number {
  return Date.UTC(date.getFullYear(), date.getMonth(), date.getDate());
}
