import { createContext } from "react";

import type {
  ProfileSettings,
  ProfileSettingsInput,
} from "@/features/profile-settings/domain/entities/profile-settings";

export interface ProfileSettingsContextValue {
  /** Always non-null: defaults until the active profile's row loads. */
  settings: ProfileSettings;
  /** True while the initial per-profile load is in flight. */
  isLoading: boolean;
  /** True while an update is being persisted. */
  isSubmitting: boolean;
  error: string | null;
  /** Merge a partial change into the current settings and persist it. */
  update: (input: ProfileSettingsInput) => Promise<boolean>;
  clearError: () => void;
}

export const ProfileSettingsContext =
  createContext<ProfileSettingsContextValue | null>(null);
