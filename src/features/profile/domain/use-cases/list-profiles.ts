import type { ProfileList } from "@/features/profile/domain/entities/profile";
import type { ProfileRepository } from "@/features/profile/domain/repositories/profile-repository";

export type ListProfilesUseCase = () => Promise<ProfileList>;

export function makeListProfilesUseCase(
  repo: ProfileRepository,
): ListProfilesUseCase {
  return () => repo.list();
}
