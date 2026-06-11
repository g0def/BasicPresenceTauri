import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type {
  WorkDayDto,
  WorkDayScheduleDto,
  WorkEntryDto,
} from "@/features/work-hours/data/dto/work-hours.dto";
import { toWorkEntry } from "@/features/work-hours/data/mappers/work-hours.mapper";
import type {
  WorkDay,
  WorkDaySchedule,
  WorkEntry,
  WorkEntryInput,
} from "@/features/work-hours/domain/entities/work-hours";
import type { WorkEntryRepository } from "@/features/work-hours/domain/repositories/work-entry-repository";

function toSchedule(dto: WorkDayScheduleDto): WorkDaySchedule {
  return { startMinutes: dto.startMinutes, endMinutes: dto.endMinutes };
}

/** WorkEntryRepository implementation backed by Tauri IPC commands. */
export class TauriWorkEntryRepository implements WorkEntryRepository {
  async listByPresence(presenceId: string): Promise<WorkEntry[]> {
    const dtos = await invoke<WorkEntryDto[]>(COMMANDS.getWorkEntries, {
      presenceId,
    });
    return dtos.map(toWorkEntry);
  }

  async replaceForPresence(
    presenceId: string,
    entries: WorkEntryInput[],
  ): Promise<WorkDay> {
    const dto = await invoke<WorkDayDto>(COMMANDS.setWorkEntries, {
      presenceId,
      entries: entries.map((e) => ({
        title: e.title,
        description: e.description ?? null,
        minutes: e.minutes,
        color: e.color,
      })),
    });
    return {
      entries: dto.entries.map(toWorkEntry),
      schedule: dto.schedule ? toSchedule(dto.schedule) : null,
    };
  }

  async getSchedule(presenceId: string): Promise<WorkDaySchedule | null> {
    const dto = await invoke<WorkDayScheduleDto | null>(
      COMMANDS.getWorkSchedule,
      { presenceId },
    );
    return dto ? toSchedule(dto) : null;
  }

  async setSchedule(
    presenceId: string,
    startMinutes: number,
  ): Promise<WorkDaySchedule> {
    const dto = await invoke<WorkDayScheduleDto>(COMMANDS.setWorkSchedule, {
      presenceId,
      startMinutes,
    });
    return toSchedule(dto);
  }
}
