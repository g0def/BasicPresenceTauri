import type { ProfileSettings } from "@/features/profile-settings/domain/entities/profile-settings";

export interface ProfileSettingsRepository {
  /** Load the profile's settings (backend returns lazy defaults if no row). */
  get(profileId: string): Promise<ProfileSettings>;
  /** Persist the full settings row; returns the canonical stored value. */
  set(profileId: string, settings: ProfileSettings): Promise<ProfileSettings>;
}
