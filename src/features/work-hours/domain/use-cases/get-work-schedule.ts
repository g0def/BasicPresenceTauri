import type { WorkDaySchedule } from "@/features/work-hours/domain/entities/work-hours";
import type { WorkEntryRepository } from "@/features/work-hours/domain/repositories/work-entry-repository";

export type GetWorkScheduleUseCase = (
  presenceId: string,
) => Promise<WorkDaySchedule | null>;

export function makeGetWorkScheduleUseCase(
  repo: WorkEntryRepository,
): GetWorkScheduleUseCase {
  return (presenceId) => repo.getSchedule(presenceId);
}
