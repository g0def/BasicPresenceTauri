import type {
  PresenceDto,
  PresenceTripDto,
} from "@/features/presence/data/dto/presence.dto";
import type {
  Presence,
  PresenceTrip,
} from "@/features/presence/domain/entities/presence";

export function toPresence(dto: PresenceDto): Presence {
  return {
    id: dto.id,
    profileId: dto.profileId,
    day: dto.day,
    type: dto.type,
    // Keep `null` for not-computed days (imports) — never coerce to a real 0.
    co2Kg: dto.co2Kg ?? null,
    isEstimated: dto.isEstimated ?? false,
    createdAt: dto.createdAt,
    updatedAt: dto.updatedAt,
  };
}

/** Map a persisted trip snapshot to the editable shape used by the day picker. */
export function toPresenceTrip(dto: PresenceTripDto): PresenceTrip {
  return {
    modeId: dto.modeId,
    distanceKm: dto.distanceKm,
    roundTrip: dto.roundTrip,
    occupants: dto.occupants,
  };
}
