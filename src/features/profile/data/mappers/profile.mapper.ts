import type { ProfileDto } from "@/features/profile/data/dto/profile.dto";
import type { Profile } from "@/entities/profile/model/profile";

export function toProfile(dto: ProfileDto): Profile {
  return {
    id: dto.id,
    firstName: dto.firstName,
    lastName: dto.lastName,
    enterprise: dto.enterprise,
    poste: dto.poste ?? null,
    createdAt: dto.createdAt,
    updatedAt: dto.updatedAt,
  };
}
