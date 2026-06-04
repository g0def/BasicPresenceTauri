import { useCallback, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

import { isAppError } from "@/core/errors";
import { TauriProfileRepository } from "@/features/profile/data/repositories/tauri-profile.repository";
import type {
  CreateProfileInput,
  Profile,
  UpdateProfileInput,
} from "@/features/profile/domain/entities/profile";
import { makeCreateProfileUseCase } from "@/features/profile/domain/use-cases/create-profile";
import { makeDeleteProfileUseCase } from "@/features/profile/domain/use-cases/delete-profile";
import { makeListProfilesUseCase } from "@/features/profile/domain/use-cases/list-profiles";
import { makeSetActiveProfileUseCase } from "@/features/profile/domain/use-cases/set-active-profile";
import { makeUpdateProfileUseCase } from "@/features/profile/domain/use-cases/update-profile";
import {
  ProfileContext,
  type ProfileContextValue,
} from "@/features/profile/presentation/context/profile-context";

// Composition root for the profile feature: wire the repository to the use cases.
const repo = new TauriProfileRepository();
const listProfilesUseCase = makeListProfilesUseCase(repo);
const createProfileUseCase = makeCreateProfileUseCase(repo);
const updateProfileUseCase = makeUpdateProfileUseCase(repo);
const deleteProfileUseCase = makeDeleteProfileUseCase(repo);
const setActiveProfileUseCase = makeSetActiveProfileUseCase(repo);

interface ProfileProviderProps {
  children: ReactNode;
  /** Called when a command reports a locked vault (SESSION_EXPIRED). The auth
   * feature owns the session, so the handler is injected by the composition
   * root (App) rather than imported here — keeps the feature self-contained. */
  onSessionExpired?: () => void;
}

export function ProfileProvider({
  children,
  onSessionExpired,
}: ProfileProviderProps) {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [activeProfileId, setActiveProfileId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // A locked vault surfaces as SESSION_EXPIRED — drop the (now stale) session.
  const handleError = useCallback(
    (e: unknown) => {
      if (isAppError(e) && e.code === "SESSION_EXPIRED") {
        onSessionExpired?.();
        return;
      }
      setError(isAppError(e) ? e.message : "Une erreur est survenue");
    },
    [onSessionExpired],
  );

  const refresh = useCallback(async () => {
    const list = await listProfilesUseCase();
    setProfiles(list.profiles);
    setActiveProfileId(list.activeProfileId);
  }, []);

  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    listProfilesUseCase()
      .then((list) => {
        if (cancelled) return;
        setProfiles(list.profiles);
        setActiveProfileId(list.activeProfileId);
      })
      .catch((e) => {
        if (!cancelled) handleError(e);
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [handleError]);

  const createProfile = useCallback(
    async (input: CreateProfileInput): Promise<boolean> => {
      setIsSubmitting(true);
      setError(null);
      try {
        await createProfileUseCase(input);
        await refresh();
        return true;
      } catch (e) {
        handleError(e);
        return false;
      } finally {
        setIsSubmitting(false);
      }
    },
    [refresh, handleError],
  );

  const updateProfile = useCallback(
    async (input: UpdateProfileInput): Promise<boolean> => {
      setIsSubmitting(true);
      setError(null);
      try {
        await updateProfileUseCase(input);
        await refresh();
        return true;
      } catch (e) {
        handleError(e);
        return false;
      } finally {
        setIsSubmitting(false);
      }
    },
    [refresh, handleError],
  );

  const deleteProfile = useCallback(
    async (id: string): Promise<void> => {
      setIsSubmitting(true);
      setError(null);
      try {
        await deleteProfileUseCase(id);
        await refresh();
      } catch (e) {
        handleError(e);
      } finally {
        setIsSubmitting(false);
      }
    },
    [refresh, handleError],
  );

  const setActiveProfile = useCallback(
    async (id: string): Promise<void> => {
      const previous = activeProfileId;
      setActiveProfileId(id); // optimistic
      try {
        await setActiveProfileUseCase(id);
      } catch (e) {
        setActiveProfileId(previous);
        handleError(e);
      }
    },
    [activeProfileId, handleError],
  );

  const clearError = useCallback(() => setError(null), []);

  const activeProfile = useMemo(
    () => profiles.find((p) => p.id === activeProfileId) ?? null,
    [profiles, activeProfileId],
  );

  const value = useMemo<ProfileContextValue>(
    () => ({
      profiles,
      activeProfile,
      isLoading,
      isSubmitting,
      error,
      createProfile,
      updateProfile,
      deleteProfile,
      setActiveProfile,
      clearError,
    }),
    [
      profiles,
      activeProfile,
      isLoading,
      isSubmitting,
      error,
      createProfile,
      updateProfile,
      deleteProfile,
      setActiveProfile,
      clearError,
    ],
  );

  return (
    <ProfileContext.Provider value={value}>{children}</ProfileContext.Provider>
  );
}
