import { TauriWorkEntryRepository } from "@/features/work-hours/data/repositories/tauri-work-entry.repository";
import { makeGetWorkEntriesUseCase } from "@/features/work-hours/domain/use-cases/get-work-entries";
import { makeGetWorkScheduleUseCase } from "@/features/work-hours/domain/use-cases/get-work-schedule";
import { makeSetWorkEntriesUseCase } from "@/features/work-hours/domain/use-cases/set-work-entries";
import { makeSetWorkScheduleUseCase } from "@/features/work-hours/domain/use-cases/set-work-schedule";

/**
 * Work-entry use cases wired to the Tauri repository, for one-off reads/writes
 * of a day's schedule and tasks outside the per-day editor hook (e.g. the
 * day-copy clipboard). The composition lives in the work-hours *presentation*
 * boundary so other features depend on it here rather than reaching into the
 * work-hours `data/` layer.
 */
const repo = new TauriWorkEntryRepository();

export const getWorkEntries = makeGetWorkEntriesUseCase(repo);
export const setWorkEntries = makeSetWorkEntriesUseCase(repo);
export const getWorkSchedule = makeGetWorkScheduleUseCase(repo);
export const setWorkSchedule = makeSetWorkScheduleUseCase(repo);
