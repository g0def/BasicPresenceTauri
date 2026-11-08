import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import { toTaskPreset } from "@/features/work-hours/data/mappers/work-hours.mapper";
import type {
  TaskPreset,
  TaskPresetInput,
} from "@/entities/work-day/model/work-hours";
import type { TaskPresetRepository } from "@/features/work-hours/domain/repositories/task-preset-repository";

/** TaskPresetRepository implementation backed by Tauri IPC commands. */
export class TauriTaskPresetRepository implements TaskPresetRepository {
  async listByProfile(profileId: string): Promise<TaskPreset[]> {
    const dtos = await invoke(COMMANDS.listTaskPresets, {
      profileId,
    });
    return dtos.map(toTaskPreset);
  }

  async create(profileId: string, input: TaskPresetInput): Promise<TaskPreset> {
    const dto = await invoke(COMMANDS.createTaskPreset, {
      profileId,
      title: input.title,
      description: input.description ?? null,
      defaultMinutes: input.defaultMinutes,
      color: input.color,
    });
    return toTaskPreset(dto);
  }

  async update(id: string, input: TaskPresetInput): Promise<TaskPreset> {
    const dto = await invoke(COMMANDS.updateTaskPreset, {
      id,
      title: input.title,
      description: input.description ?? null,
      defaultMinutes: input.defaultMinutes,
      color: input.color,
    });
    return toTaskPreset(dto);
  }

  async remove(id: string): Promise<void> {
    await invoke(COMMANDS.deleteTaskPreset, { id });
  }
}
