import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type { TaskPresetDto } from "@/features/work-hours/data/dto/work-hours.dto";
import { toTaskPreset } from "@/features/work-hours/data/mappers/work-hours.mapper";
import type {
  TaskPreset,
  TaskPresetInput,
} from "@/features/work-hours/domain/entities/work-hours";
import type { TaskPresetRepository } from "@/features/work-hours/domain/repositories/task-preset-repository";

/** TaskPresetRepository implementation backed by Tauri IPC commands. */
export class TauriTaskPresetRepository implements TaskPresetRepository {
  async listByProfile(profileId: string): Promise<TaskPreset[]> {
    const dtos = await invoke<TaskPresetDto[]>(COMMANDS.listTaskPresets, {
      profileId,
    });
    return dtos.map(toTaskPreset);
  }

  async create(profileId: string, input: TaskPresetInput): Promise<TaskPreset> {
    const dto = await invoke<TaskPresetDto>(COMMANDS.createTaskPreset, {
      profileId,
      title: input.title,
      description: input.description ?? null,
      defaultMinutes: input.defaultMinutes,
      color: input.color,
    });
    return toTaskPreset(dto);
  }

  async update(id: string, input: TaskPresetInput): Promise<TaskPreset> {
    const dto = await invoke<TaskPresetDto>(COMMANDS.updateTaskPreset, {
      id,
      title: input.title,
      description: input.description ?? null,
      defaultMinutes: input.defaultMinutes,
      color: input.color,
    });
    return toTaskPreset(dto);
  }

  async remove(id: string): Promise<void> {
    await invoke<void>(COMMANDS.deleteTaskPreset, { id });
  }
}
