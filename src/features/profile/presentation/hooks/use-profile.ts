import { useContext } from "react";

import {
  ProfileContext,
  type ProfileContextValue,
} from "@/features/profile/presentation/context/profile-context";

export function useProfile(): ProfileContextValue {
  const ctx = useContext(ProfileContext);
  if (!ctx) {
    throw new Error("useProfile must be used within <ProfileProvider>");
  }
  return ctx;
}
