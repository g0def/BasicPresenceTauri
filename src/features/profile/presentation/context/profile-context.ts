import { createContext } from "react";

import type {
  CreateProfileInput,
  Profile,
  UpdateProfileInput,
} from "@/features/profile/domain/entities/profile";

export interface ProfileContextValue {
  profiles: Profile[];
  /** The currently selected profile (shown in the header), or `null`. */
  activeProfile: Profile | null;
  /** True while the initial profile list is loading. */
  isLoading: boolean;
  /** True while a create/update/delete is in flight. */
  isSubmitting: boolean;
  error: string | null;
  /** Returns `true` on success so callers (dialogs) can close themselves. */
  createProfile: (input: CreateProfileInput) => Promise<boolean>;
  updateProfile: (input: UpdateProfileInput) => Promise<boolean>;
  deleteProfile: (id: string) => Promise<void>;
  setActiveProfile: (id: string) => Promise<void>;
  clearError: () => void;
}

export const ProfileContext = createContext<ProfileContextValue | null>(null);
