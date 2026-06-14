import { useCallback, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { startOfMonth } from "date-fns";

import { isAppError } from "@/core/errors";
import { TauriPresenceRepository } from "@/features/presence/data/repositories/tauri-presence.repository";
import type {
  ImportPresenceEntry,
  ImportSummary,
  Presence,
  PresenceTrip,
  PresenceType,
} from "@/features/presence/domain/entities/presence";
import { makeDeletePresenceUseCase } from "@/features/presence/domain/use-cases/delete-presence";
import { makeGetPresenceTripsUseCase } from "@/features/presence/domain/use-cases/get-presence-trips";
import { makeImportPresencesUseCase } from "@/features/presence/domain/use-cases/import-presences";
import { makeListPresencesUseCase } from "@/features/presence/domain/use-cases/list-presences";
import { makeSetPresenceUseCase } from "@/features/presence/domain/use-cases/set-presence";
import {
  PresenceContext,
  type PresenceContextValue,
} from "@/features/presence/presentation/context/presence-context";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";

// Composition root for the presence feature: wire the repository to the use cases.
const repo = new TauriPresenceRepository();
const listPresencesUseCase = makeListPresencesUseCase(repo);
const setPresenceUseCase = makeSetPresenceUseCase(repo);
const deletePresenceUseCase = makeDeletePresenceUseCase(repo);
const importPresencesUseCase = makeImportPresencesUseCase(repo);
const getPresenceTripsUseCase = makeGetPresenceTripsUseCase(repo);

interface PresenceProviderProps {
  children: ReactNode;
  /** Called when a command reports a locked vault (SESSION_EXPIRED). Injected
   * by the composition root (App) so the feature stays independent of auth. */
  onSessionExpired?: () => void;
}

export function PresenceProvider({
  children,
  onSessionExpired,
}: PresenceProviderProps) {
  // Presences are scoped to the active profile (owned by the profile feature).
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? null;

  const [presences, setPresences] = useState<Presence[]>([]);
  const [currentMonth, setCurrentMonth] = useState(() =>
    startOfMonth(new Date()),
  );
  const [isLoading, setIsLoading] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

  // Reload presences whenever the active profile changes.
  useEffect(() => {
    if (!profileId) {
      setPresences([]);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    listPresencesUseCase(profileId)
      .then((list) => {
        if (!cancelled) setPresences(list);
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
  }, [profileId, handleError]);

  const setPresence = useCallback(
    async (
      day: number,
      type: PresenceType,
      trips?: PresenceTrip[],
    ): Promise<boolean> => {
      if (!profileId) return false;
      setIsSubmitting(true);
      setError(null);
      try {
        const saved = await setPresenceUseCase({ profileId, day, type, trips });
        // Upsert by day: drop any previous record for that day, keep the rest.
        setPresences((prev) => [
          ...prev.filter((p) => p.day !== saved.day),
          saved,
        ]);
        return true;
      } catch (e) {
        handleError(e);
        return false;
      } finally {
        setIsSubmitting(false);
      }
    },
    [profileId, handleError],
  );

  const getPresenceTrips = useCallback(
    async (presenceId: string): Promise<PresenceTrip[]> => {
      try {
        return await getPresenceTripsUseCase(presenceId);
      } catch (e) {
        handleError(e);
        return [];
      }
    },
    [handleError],
  );

  const deletePresence = useCallback(
    async (id: string): Promise<boolean> => {
      setIsSubmitting(true);
      setError(null);
      try {
        await deletePresenceUseCase(id);
        setPresences((prev) => prev.filter((p) => p.id !== id));
        return true;
      } catch (e) {
        handleError(e);
        return false;
      } finally {
        setIsSubmitting(false);
      }
    },
    [handleError],
  );

  // Re-pull the whole list from the backend. Used after a bulk import, where a
  // single re-list is simpler (and safer) than reconciling many rows by hand.
  const reload = useCallback(async (): Promise<void> => {
    if (!profileId) {
      setPresences([]);
      return;
    }
    setIsLoading(true);
    try {
      setPresences(await listPresencesUseCase(profileId));
    } catch (e) {
      handleError(e);
    } finally {
      setIsLoading(false);
    }
  }, [profileId, handleError]);

  const importPresences = useCallback(
    async (
      entries: ImportPresenceEntry[],
      replaceExisting: boolean,
    ): Promise<ImportSummary | null> => {
      if (!profileId) return null;
      setIsSubmitting(true);
      setError(null);
      try {
        const summary = await importPresencesUseCase({
          profileId,
          entries,
          replaceExisting,
        });
        await reload();
        return summary;
      } catch (e) {
        handleError(e);
        return null;
      } finally {
        setIsSubmitting(false);
      }
    },
    [profileId, reload, handleError],
  );

  const clearError = useCallback(() => setError(null), []);

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
      importPresences,
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
      importPresences,
      clearError,
    ],
  );

  return (
    <PresenceContext.Provider value={value}>
      {children}
    </PresenceContext.Provider>
  );
}
