import { useCallback, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useContainer } from "@/core/di";
import { errorMessage, isSessionExpired, useCommand } from "@/core/query";
import type {
  CreateProfileInput,
  Profile,
  ProfileList,
  UpdateProfileInput,
} from "@/entities/profile/model/profile";
import { makeCreateProfileUseCase } from "@/features/profile/domain/use-cases/create-profile";
import { makeDeleteProfileUseCase } from "@/features/profile/domain/use-cases/delete-profile";
import { makeListProfilesUseCase } from "@/features/profile/domain/use-cases/list-profiles";
import { makeSetActiveProfileUseCase } from "@/features/profile/domain/use-cases/set-active-profile";
import { makeUpdateProfileUseCase } from "@/features/profile/domain/use-cases/update-profile";
import {
  ProfileContext,
  type ProfileContextValue,
} from "@/entities/profile/model/profile-context";
import { profileKeys } from "@/features/profile/presentation/query-keys";

const NO_PROFILES: Profile[] = [];

/**
 * Profiles of the device owner + the active one. Server state lives in the
 * query cache (`profileKeys.all`); a locked vault (SESSION_EXPIRED) is handled
 * globally by the query client.
 */
export function ProfileProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const { profileRepository } = useContainer();
  const useCases = useMemo(
    () => ({
      list: makeListProfilesUseCase(profileRepository),
      create: makeCreateProfileUseCase(profileRepository),
      update: makeUpdateProfileUseCase(profileRepository),
      remove: makeDeleteProfileUseCase(profileRepository),
      setActive: makeSetActiveProfileUseCase(profileRepository),
    }),
    [profileRepository],
  );
  const queryClient = useQueryClient();
  const { run, isPending: isSubmitting } = useCommand();
  const [error, setError] = useState<string | null>(null);

  const listQuery = useQuery({
    queryKey: profileKeys.all,
    queryFn: () => useCases.list(),
  });

  const reportError = useCallback(
    (e: unknown) => {
      if (!isSessionExpired(e)) setError(errorMessage(e, t("errors.generic")));
    },
    [t],
  );

  useEffect(() => {
    if (listQuery.error) reportError(listQuery.error);
  }, [listQuery.error, reportError]);

  const refresh = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: profileKeys.all });
  }, [queryClient]);

  const mutate = useCallback(
    async (work: () => Promise<unknown>): Promise<boolean> => {
      setError(null);
      try {
        await run(work);
        await refresh();
        return true;
      } catch (e) {
        reportError(e);
        return false;
      }
    },
    [run, refresh, reportError],
  );

  const createProfile = useCallback(
    (input: CreateProfileInput) => mutate(() => useCases.create(input)),
    [mutate, useCases],
  );

  const updateProfile = useCallback(
    (input: UpdateProfileInput) => mutate(() => useCases.update(input)),
    [mutate, useCases],
  );

  const deleteProfile = useCallback(
    async (id: string): Promise<void> => {
      await mutate(() => useCases.remove(id));
    },
    [mutate, useCases],
  );

  const setActiveProfile = useCallback(
    async (id: string): Promise<void> => {
      const previous = queryClient.getQueryData<ProfileList>(profileKeys.all);
      // Optimistic: the header switches immediately.
      queryClient.setQueryData<ProfileList>(profileKeys.all, (old) =>
        old ? { ...old, activeProfileId: id } : old,
      );
      try {
        await run(() => useCases.setActive(id));
      } catch (e) {
        queryClient.setQueryData(profileKeys.all, previous);
        reportError(e);
      }
    },
    [queryClient, run, useCases, reportError],
  );

  const clearError = useCallback(() => setError(null), []);

  const profiles = listQuery.data?.profiles ?? NO_PROFILES;
  const activeProfileId = listQuery.data?.activeProfileId ?? null;
  const activeProfile = useMemo(
    () => profiles.find((p) => p.id === activeProfileId) ?? null,
    [profiles, activeProfileId],
  );

  const value = useMemo<ProfileContextValue>(
    () => ({
      profiles,
      activeProfile,
      isLoading: listQuery.isPending,
      isSubmitting,
      error,
      createProfile,
      updateProfile,
      deleteProfile,
      setActiveProfile,
      refresh,
      clearError,
    }),
    [
      profiles,
      activeProfile,
      listQuery.isPending,
      isSubmitting,
      error,
      createProfile,
      updateProfile,
      deleteProfile,
      setActiveProfile,
      refresh,
      clearError,
    ],
  );

  return (
    <ProfileContext.Provider value={value}>{children}</ProfileContext.Provider>
  );
}
