import type { DayNote } from "@/features/work-hours/domain/entities/day-note";
import type { DayNoteRepository } from "@/features/work-hours/domain/repositories/day-note-repository";

export type GetDayNoteUseCase = (presenceId: string) => Promise<DayNote>;

export function makeGetDayNoteUseCase(
  repo: DayNoteRepository,
): GetDayNoteUseCase {
  return (presenceId) => repo.get(presenceId);
}
