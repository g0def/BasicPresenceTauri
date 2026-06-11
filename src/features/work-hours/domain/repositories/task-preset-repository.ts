import type {
  TaskPreset,
  TaskPresetInput,
} from "@/features/work-hours/domain/entities/work-hours";

export interface TaskPresetRepository {
  listByProfile(profileId: string): Promise<TaskPreset[]>;
  create(profileId: string, input: TaskPresetInput): Promise<TaskPreset>;
  update(id: string, input: TaskPresetInput): Promise<TaskPreset>;
  remove(id: string): Promise<void>;
}
