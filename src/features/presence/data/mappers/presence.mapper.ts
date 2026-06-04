import type { PresenceDto } from "@/features/presence/data/dto/presence.dto";
import type { Presence } from "@/features/presence/domain/entities/presence";

export function toPresence(dto: PresenceDto): Presence {
  return {
    id: dto.id,
    profileId: dto.profileId,
    day: dto.day,
    type: dto.type,
    createdAt: dto.createdAt,
    updatedAt: dto.updatedAt,
  };
}
