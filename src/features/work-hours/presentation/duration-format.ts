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

/** Minutes since midnight → `"HH:MM"` (the `<input type="time">` format).
 * Values past 1440 wrap to the next day (stacked tasks can cross midnight). */
export function formatClockTime(minutes: number): string {
  const safe = ((Math.round(minutes) % 1440) + 1440) % 1440;
  const hours = Math.floor(safe / 60);
  const mins = safe % 60;
  return `${String(hours).padStart(2, "0")}:${String(mins).padStart(2, "0")}`;
}
