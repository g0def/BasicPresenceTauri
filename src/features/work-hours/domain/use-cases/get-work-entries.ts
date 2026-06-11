import type { WorkEntry } from "@/features/work-hours/domain/entities/work-hours";
import type { WorkEntryRepository } from "@/features/work-hours/domain/repositories/work-entry-repository";

export type GetWorkEntriesUseCase = (
  presenceId: string,
) => Promise<WorkEntry[]>;

export function makeGetWorkEntriesUseCase(
  repo: WorkEntryRepository,
): GetWorkEntriesUseCase {
  return (presenceId) => repo.listByPresence(presenceId);
}
