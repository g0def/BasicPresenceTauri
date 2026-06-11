import type {
  WorkDay,
  WorkDaySchedule,
  WorkEntry,
  WorkEntryInput,
} from "@/features/work-hours/domain/entities/work-hours";

export interface WorkEntryRepository {
  listByPresence(presenceId: string): Promise<WorkEntry[]>;
  /** Atomically replace the full entry set of a presence day. Returns the
   * canonical entries plus the schedule (its stored end recomputed). */
  replaceForPresence(
    presenceId: string,
    entries: WorkEntryInput[],
  ): Promise<WorkDay>;
  /** The day's start/end times, or `null` when never set. */
  getSchedule(presenceId: string): Promise<WorkDaySchedule | null>;
  /** Set the day's start time; the backend recomputes and stores the end. */
  setSchedule(
    presenceId: string,
    startMinutes: number,
  ): Promise<WorkDaySchedule>;
}
