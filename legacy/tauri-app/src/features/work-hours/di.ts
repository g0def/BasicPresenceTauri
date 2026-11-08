// Ports this feature takes from the DI container (see core/di).
import type { DayNoteRepository } from "@/features/work-hours/domain/repositories/day-note-repository";
import type { TaskPresetRepository } from "@/features/work-hours/domain/repositories/task-preset-repository";
import type { WorkEntryRepository } from "@/features/work-hours/domain/repositories/work-entry-repository";

declare module "@/core/di" {
  interface Container {
    taskPresetRepository: TaskPresetRepository;
    workEntryRepository: WorkEntryRepository;
    dayNoteRepository: DayNoteRepository;
  }
}
