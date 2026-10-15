import { useCallback, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { startOfMonth } from "date-fns";

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useContainer } from "@/core/di";
import { errorMessage, isSessionExpired, useCommand } from "@/core/query";
import type {
  Presence,
  PresenceTrip,
  PresenceType,
} from "@/entities/presence/model/presence";
import { makeDeletePresenceUseCase } from "@/features/presence/domain/use-cases/delete-presence";
import { makeGetPresenceTripsUseCase } from "@/features/presence/domain/use-cases/get-presence-trips";
import { makeListPresencesUseCase } from "@/features/presence/domain/use-cases/list-presences";
import { makeSetPresenceUseCase } from "@/features/presence/domain/use-cases/set-presence";
import {
  PresenceContext,
  type PresenceContextValue,
} from "@/entities/presence/model/presence-context";
import { presenceKeys } from "@/features/presence/presentation/query-keys";
import { useProfile } from "@/entities/profile/model/use-profile";

const NO_PRESENCES: Presence[] = [];

export function PresenceProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  // Presences are scoped to the active profile (owned by the profile feature).
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? null;

  const { presenceRepository } = useContainer();
  const useCases = useMemo(
    () => ({
      list: makeListPresencesUseCase(presenceRepository),
      set: makeSetPresenceUseCase(presenceRepository),
      remove: makeDeletePresenceUseCase(presenceRepository),
      trips: makeGetPresenceTripsUseCase(presenceRepository),
    }),
    [presenceRepository],
  );
  const queryClient = useQueryClient();
  const { run, isPending: isSubmitting } = useCommand();
  const [currentMonth, setCurrentMonth] = useState(() =>
    startOfMonth(new Date()),
  );
  const [error, setError] = useState<string | null>(null);

  const reportError = useCallback(
    (e: unknown) => {
      if (!isSessionExpired(e)) setError(errorMessage(e, t("errors.generic")));
    },
    [t],
  );

  // Reloads whenever the active profile changes (the key carries it).
  const key = presenceKeys.byProfile(profileId);
  const query = useQuery({
    queryKey: key,
    enabled: profileId !== null,
    queryFn: () => useCases.list(profileId as string),
  });
  const presences = (profileId && query.data) || NO_PRESENCES;

  useEffect(() => {
    if (query.error) reportError(query.error);
  }, [query.error, reportError]);

  const setPresence = useCallback(
    async (
      day: number,
      type: PresenceType,
      trips?: PresenceTrip[],
    ): Promise<Presence | null> => {
      if (!profileId) return null;
      setError(null);
      try {
        const saved = await run(() =>
          useCases.set({ profileId, day, type, trips }),
        );
        // Upsert by day: drop any previous record for that day, keep the rest.
        queryClient.setQueryData<Presence[]>(
          presenceKeys.byProfile(profileId),
          (prev = []) => [...prev.filter((p) => p.day !== saved.day), saved],
        );
        queryClient.removeQueries({ queryKey: presenceKeys.trips(saved.id) });
        return saved;
      } catch (e) {
        reportError(e);
        return null;
      }
    },
    [profileId, run, useCases, queryClient, reportError],
  );

  const getPresenceTrips = useCallback(
    async (presenceId: string): Promise<PresenceTrip[]> => {
      try {
        return await queryClient.fetchQuery({
          queryKey: presenceKeys.trips(presenceId),
          queryFn: () => useCases.trips(presenceId),
        });
      } catch (e) {
        reportError(e);
        return [];
      }
    },
    [queryClient, useCases, reportError],
  );

  const deletePresence = useCallback(
    async (id: string): Promise<boolean> => {
      setError(null);
      try {
        await run(() => useCases.remove(id));
        queryClient.setQueryData<Presence[]>(
          presenceKeys.byProfile(profileId),
          (prev = []) => prev.filter((p) => p.id !== id),
        );
        queryClient.removeQueries({ queryKey: presenceKeys.trips(id) });
        return true;
      } catch (e) {
        reportError(e);
        return false;
      }
    },
    [profileId, run, useCases, queryClient, reportError],
  );

  // Re-pull the whole list from the backend. Used after a bulk import or a
  // work-hours change, where a single re-list is simpler (and safer) than
  // reconciling many rows by hand.
  const reload = useCallback(async (): Promise<void> => {
    if (!profileId) return;
    await queryClient.invalidateQueries({
      queryKey: presenceKeys.byProfile(profileId),
    });
  }, [profileId, queryClient]);

  const clearError = useCallback(() => setError(null), []);
  const isLoading = profileId !== null && (query.isPending || query.isFetching);

  const presencesByDay = useMemo(() => {
    const map = new Map<number, Presence>();
    for (const p of presences) map.set(p.day, p);
    return map;
  }, [presences]);

  const value = useMemo<PresenceContextValue>(
    () => ({
      presencesByDay,
      currentMonth,
      setCurrentMonth,
      reload,
      isLoading,
      isSubmitting,
      error,
      setPresence,
      deletePresence,
      getPresenceTrips,
      clearError,
    }),
    [
      presencesByDay,
      currentMonth,
      reload,
      isLoading,
      isSubmitting,
      error,
      setPresence,
      deletePresence,
      getPresenceTrips,
      clearError,
    ],
  );

  return (
    <PresenceContext.Provider value={value}>
      {children}
    </PresenceContext.Provider>
  );
}
