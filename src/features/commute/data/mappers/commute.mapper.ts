import type {
  CommuteDto,
  EmissionFactorDto,
} from "@/features/commute/data/dto/commute.dto";
import {
  COMMUTE_CATEGORIES,
  type Commute,
  type CommuteCategory,
  type EmissionFactor,
} from "@/features/commute/domain/entities/commute";

function toCategory(value: string): CommuteCategory {
  return (COMMUTE_CATEGORIES as readonly string[]).includes(value)
    ? (value as CommuteCategory)
    : "other";
}

export function toEmissionFactor(dto: EmissionFactorDto): EmissionFactor {
  return {
    modeId: dto.modeId,
    label: dto.label,
    category: toCategory(dto.category),
    unit: dto.unit,
    isParam: dto.isParam,
  };
}

export function toCommute(dto: CommuteDto): Commute {
  return {
    id: dto.id,
    profileId: dto.profileId,
    name: dto.name,
    roundTrip: dto.roundTrip,
    segments: dto.segments.map((s) => ({
      modeId: s.modeId,
      distanceKm: s.distanceKm,
      occupants: s.occupants,
    })),
    co2Kg: dto.co2Kg ?? undefined,
    createdAt: dto.createdAt,
    updatedAt: dto.updatedAt,
  };
}
