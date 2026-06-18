import { useContext } from "react";

import {
  ProfileSettingsContext,
  type ProfileSettingsContextValue,
} from "@/features/profile-settings/presentation/context/profile-settings-context";

export function useProfileSettings(): ProfileSettingsContextValue {
  const ctx = useContext(ProfileSettingsContext);
  if (!ctx) {
    throw new Error(
      "useProfileSettings must be used within <ProfileSettingsProvider>",
    );
  }
  return ctx;
}
