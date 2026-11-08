import type { DayNote } from "@/features/work-hours/domain/entities/day-note";
import type { DayNoteRepository } from "@/features/work-hours/domain/repositories/day-note-repository";

export type SetDayNoteUseCase = (
  presenceId: string,
  markdown: string,
) => Promise<DayNote>;

export function makeSetDayNoteUseCase(
  repo: DayNoteRepository,
): SetDayNoteUseCase {
  return (presenceId, markdown) => repo.set(presenceId, markdown);
}
