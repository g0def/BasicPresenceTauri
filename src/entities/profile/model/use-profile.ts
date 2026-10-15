import { useContext } from "react";

import {
  ProfileContext,
  type ProfileContextValue,
} from "@/entities/profile/model/profile-context";

export function useProfile(): ProfileContextValue {
  const ctx = useContext(ProfileContext);
  if (!ctx) {
    throw new Error("useProfile must be used within <ProfileProvider>");
  }
  return ctx;
}
