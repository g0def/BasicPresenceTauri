import type { DayNote } from "@/features/work-hours/domain/entities/day-note";

export interface DayNoteRepository {
  /** The day's note (empty markdown/html when unset). */
  get(presenceId: string): Promise<DayNote>;
  /** Persist the day's Markdown note (a blank note clears it); returns the
   * stored Markdown plus its freshly rendered, sanitized HTML. */
  set(presenceId: string, markdown: string): Promise<DayNote>;
}
