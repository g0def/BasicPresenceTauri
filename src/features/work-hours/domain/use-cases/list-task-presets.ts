import type { TaskPreset } from "@/features/work-hours/domain/entities/work-hours";
import type { TaskPresetRepository } from "@/features/work-hours/domain/repositories/task-preset-repository";

export type ListTaskPresetsUseCase = (
  profileId: string,
) => Promise<TaskPreset[]>;

export function makeListTaskPresetsUseCase(
  repo: TaskPresetRepository,
): ListTaskPresetsUseCase {
  return (profileId) => repo.listByProfile(profileId);
}
