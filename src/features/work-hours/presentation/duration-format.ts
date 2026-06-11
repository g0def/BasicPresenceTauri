/**
 * Compact duration label on the h/min convention shared by FR and EN:
 * `45 → "45min"`, `60 → "1h"`, `65 → "1h05"`, `450 → "7h30"`.
 */
export function formatMinutes(minutes: number): string {
  const safe = Math.max(0, Math.round(minutes));
  const hours = Math.floor(safe / 60);
  const mins = safe % 60;
  if (hours === 0) return `${mins}min`;
  if (mins === 0) return `${hours}h`;
  return `${hours}h${String(mins).padStart(2, "0")}`;
}

/**
 * Lenient clock-time parsing for free typing: `"8"` → 480, and `"8:30"`,
 * `"8h30"`, `"8.30"`, `"830"`, `"0830"` all → 510. Returns `null` when the
 * text is not a valid time of day.
 */
export function parseClockTime(raw: string): number | null {
  const match = /^(\d{1,2})(?:\s*[:h.,]\s*(\d{0,2})|(\d{2}))?$/.exec(
    raw.trim().toLowerCase(),
  );
  if (!match) return null;
  const hours = Number(match[1]);
  const minutes = Number(match[2] || match[3] || "0");
  if (hours > 23 || minutes > 59) return null;
  return hours * 60 + minutes;
}

/** Minutes since midnight → `"HH:MM"` (the `<input type="time">` format).
 * Values past 1440 wrap to the next day (stacked tasks can cross midnight). */
export function formatClockTime(minutes: number): string {
  const safe = ((Math.round(minutes) % 1440) + 1440) % 1440;
  const hours = Math.floor(safe / 60);
  const mins = safe % 60;
  return `${String(hours).padStart(2, "0")}:${String(mins).padStart(2, "0")}`;
}
