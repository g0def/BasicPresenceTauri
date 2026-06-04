import type { ProfileRepository } from "@/features/profile/domain/repositories/profile-repository";

export type DeleteProfileUseCase = (id: string) => Promise<void>;

export function makeDeleteProfileUseCase(
  repo: ProfileRepository,
): DeleteProfileUseCase {
  return (id) => repo.remove(id);
}
