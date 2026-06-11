import type {
  TaskPresetDto,
  WorkEntryDto,
} from "@/features/work-hours/data/dto/work-hours.dto";
import type {
  TaskPreset,
  WorkEntry,
} from "@/features/work-hours/domain/entities/work-hours";

export function toTaskPreset(dto: TaskPresetDto): TaskPreset {
  return {
    id: dto.id,
    profileId: dto.profileId,
    title: dto.title,
    description: dto.description,
    defaultMinutes: dto.defaultMinutes,
    color: dto.color,
    createdAt: dto.createdAt,
    updatedAt: dto.updatedAt,
  };
}

export function toWorkEntry(dto: WorkEntryDto): WorkEntry {
  return {
    id: dto.id,
    title: dto.title,
    description: dto.description,
    minutes: dto.minutes,
    color: dto.color,
    position: dto.position,
  };
}
