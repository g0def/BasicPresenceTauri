import type { WorkDaySchedule } from "@/features/work-hours/domain/entities/work-hours";
import type { WorkEntryRepository } from "@/features/work-hours/domain/repositories/work-entry-repository";

export type SetWorkScheduleUseCase = (
  presenceId: string,
  startMinutes: number,
) => Promise<WorkDaySchedule>;

export function makeSetWorkScheduleUseCase(
  repo: WorkEntryRepository,
): SetWorkScheduleUseCase {
  return (presenceId, startMinutes) =>
    repo.setSchedule(presenceId, startMinutes);
}
