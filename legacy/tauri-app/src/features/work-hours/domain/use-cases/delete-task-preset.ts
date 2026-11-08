import type { TaskPresetRepository } from "@/features/work-hours/domain/repositories/task-preset-repository";

export type DeleteTaskPresetUseCase = (id: string) => Promise<void>;

export function makeDeleteTaskPresetUseCase(
  repo: TaskPresetRepository,
): DeleteTaskPresetUseCase {
  return (id) => repo.remove(id);
}
