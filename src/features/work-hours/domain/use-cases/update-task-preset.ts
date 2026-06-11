import type {
  TaskPreset,
  TaskPresetInput,
} from "@/features/work-hours/domain/entities/work-hours";
import type { TaskPresetRepository } from "@/features/work-hours/domain/repositories/task-preset-repository";

export type UpdateTaskPresetUseCase = (
  id: string,
  input: TaskPresetInput,
) => Promise<TaskPreset>;

export function makeUpdateTaskPresetUseCase(
  repo: TaskPresetRepository,
): UpdateTaskPresetUseCase {
  return (id, input) => repo.update(id, input);
}
