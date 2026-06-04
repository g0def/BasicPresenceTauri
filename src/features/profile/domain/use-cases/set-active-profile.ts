import type { ProfileRepository } from "@/features/profile/domain/repositories/profile-repository";

export type SetActiveProfileUseCase = (id: string) => Promise<void>;

export function makeSetActiveProfileUseCase(
  repo: ProfileRepository,
): SetActiveProfileUseCase {
  return (id) => repo.setActive(id);
}
