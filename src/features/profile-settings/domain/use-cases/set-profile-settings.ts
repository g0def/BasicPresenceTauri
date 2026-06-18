import type { ProfileSettings } from "@/features/profile-settings/domain/entities/profile-settings";
import type { ProfileSettingsRepository } from "@/features/profile-settings/domain/repositories/profile-settings-repository";

export type SetProfileSettingsUseCase = (
  profileId: string,
  settings: ProfileSettings,
) => Promise<ProfileSettings>;

export function makeSetProfileSettingsUseCase(
  repo: ProfileSettingsRepository,
): SetProfileSettingsUseCase {
  return (profileId, settings) => repo.set(profileId, settings);
}
