import type {
  TaskPreset,
  TaskPresetInput,
} from "@/features/work-hours/domain/entities/work-hours";
import type { TaskPresetRepository } from "@/features/work-hours/domain/repositories/task-preset-repository";

export type CreateTaskPresetUseCase = (
  profileId: string,
  input: TaskPresetInput,
) => Promise<TaskPreset>;

export function makeCreateTaskPresetUseCase(
  repo: TaskPresetRepository,
): CreateTaskPresetUseCase {
  return (profileId, input) => repo.create(profileId, input);
}
