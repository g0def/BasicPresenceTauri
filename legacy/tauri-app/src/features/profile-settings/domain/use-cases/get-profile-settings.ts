import type { ProfileSettings } from "@/features/profile-settings/domain/entities/profile-settings";
import type { ProfileSettingsRepository } from "@/features/profile-settings/domain/repositories/profile-settings-repository";

export type GetProfileSettingsUseCase = (
  profileId: string,
) => Promise<ProfileSettings>;

export function makeGetProfileSettingsUseCase(
  repo: ProfileSettingsRepository,
): GetProfileSettingsUseCase {
  return (profileId) => repo.get(profileId);
}
