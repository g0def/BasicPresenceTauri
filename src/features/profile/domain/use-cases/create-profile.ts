import type {
  CreateProfileInput,
  Profile,
} from "@/features/profile/domain/entities/profile";
import type { ProfileRepository } from "@/features/profile/domain/repositories/profile-repository";

export type CreateProfileUseCase = (
  input: CreateProfileInput,
) => Promise<Profile>;

export function makeCreateProfileUseCase(
  repo: ProfileRepository,
): CreateProfileUseCase {
  return (input) => repo.create(input);
}
