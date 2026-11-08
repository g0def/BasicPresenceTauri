import type {
  Profile,
  UpdateProfileInput,
} from "@/entities/profile/model/profile";
import type { ProfileRepository } from "@/features/profile/domain/repositories/profile-repository";

export type UpdateProfileUseCase = (
  input: UpdateProfileInput,
) => Promise<Profile>;

export function makeUpdateProfileUseCase(
  repo: ProfileRepository,
): UpdateProfileUseCase {
  return (input) => repo.update(input);
}
