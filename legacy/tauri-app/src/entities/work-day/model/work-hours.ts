/** Bounds of a task duration: the slider moves on a 5-minute grid up to 8h. */
export const MIN_MINUTES = 5;
export const MAX_MINUTES = 480;
export const STEP_MINUTES = 5;

/** A reusable task preset owned by a profile. `defaultMinutes` only seeds the
 * duration when the preset is added to a day. */
export interface TaskPreset {
  id: string;
  profileId: string;
  title: string;
  description: string | null;
  defaultMinutes: number;
  /** '#RRGGBB' hex color of the donut segment. */
  color: string;
  createdAt: number;
  updatedAt: number;
}

/** Fields supplied when creating/updating a preset (ids/timestamps backend-managed). */
export interface TaskPresetInput {
  title: string;
  description?: string;
  defaultMinutes: number;
  color: string;
}

/** One task of a work day. Entries are duration-stacked in `position` order
 * (no explicit start times); the day total is the sum of `minutes`. */
export interface WorkEntry {
  id: string;
  title: string;
  description: string | null;
  minutes: number;
  color: string;
  position: number;
}

/** An entry sent to `set_work_entries` (position = array order, ids regenerated). */
export interface WorkEntryInput {
  title: string;
  description?: string;
  minutes: number;
  color: string;
}

export function toWorkEntryInput(e: WorkEntry): WorkEntryInput {
  return {
    title: e.title,
    description: e.description ?? undefined,
    minutes: e.minutes,
    color: e.color,
  };
}

/** Start/end clock times of the day, in minutes since midnight, as stored in
 * the vault. The end is recomputed backend-side on every save (start + sum of
 * the entries' durations) and may exceed 1440 when tasks run past midnight.
 * The start anchors each task to a real clock time (entry N starts at start +
 * sum of previous durations). */
export interface WorkDaySchedule {
  startMinutes: number;
  endMinutes: number;
}

/** Result of a day's entries save: canonical entries + recomputed schedule. */
export interface WorkDay {
  entries: WorkEntry[];
  schedule: WorkDaySchedule | null;
}
