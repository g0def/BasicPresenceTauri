import type {
  WorkDay,
  WorkEntryInput,
} from "@/features/work-hours/domain/entities/work-hours";
import type { WorkEntryRepository } from "@/features/work-hours/domain/repositories/work-entry-repository";

export type SetWorkEntriesUseCase = (
  presenceId: string,
  entries: WorkEntryInput[],
) => Promise<WorkDay>;

export function makeSetWorkEntriesUseCase(
  repo: WorkEntryRepository,
): SetWorkEntriesUseCase {
  return (presenceId, entries) => repo.replaceForPresence(presenceId, entries);
}
