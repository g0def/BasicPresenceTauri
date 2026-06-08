import { useCallback, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

import { isAppError } from "@/core/errors";
import { TauriCommuteRepository } from "@/features/commute/data/repositories/tauri-commute.repository";
import { TauriEmissionFactorRepository } from "@/features/commute/data/repositories/tauri-emission-factor.repository";
import type {
  Commute,
  CreateCommuteInput,
  EmissionFactor,
  UpdateCommuteInput,
} from "@/features/commute/domain/entities/commute";
import { makeCreateCommuteUseCase } from "@/features/commute/domain/use-cases/create-commute";
import { makeDeleteCommuteUseCase } from "@/features/commute/domain/use-cases/delete-commute";
import { makeListCommutesUseCase } from "@/features/commute/domain/use-cases/list-commutes";
import { makeListEmissionFactorsUseCase } from "@/features/commute/domain/use-cases/list-emission-factors";
import { makeUpdateCommuteUseCase } from "@/features/commute/domain/use-cases/update-commute";
import {
  CommuteContext,
  type CommuteContextValue,
} from "@/features/commute/presentation/context/commute-context";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";

// Composition root for the commute feature: wire repositories to use cases.
const commuteRepo = new TauriCommuteRepository();
const factorRepo = new TauriEmissionFactorRepository();
const listCommutesUseCase = makeListCommutesUseCase(commuteRepo);
const createCommuteUseCase = makeCreateCommuteUseCase(commuteRepo);
const updateCommuteUseCase = makeUpdateCommuteUseCase(commuteRepo);
const deleteCommuteUseCase = makeDeleteCommuteUseCase(commuteRepo);
const listEmissionFactorsUseCase = makeListEmissionFactorsUseCase(factorRepo);

interface CommuteProviderProps {
  children: ReactNode;
  /** Called when a command reports a locked vault (SESSION_EXPIRED). Injected
   * by the composition root (App) so the feature stays independent of auth. */
  onSessionExpired?: () => void;
}

export function CommuteProvider({
  children,
  onSessionExpired,
}: CommuteProviderProps) {
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? null;

  const [commutes, setCommutes] = useState<Commute[]>([]);
  const [emissionFactors, setEmissionFactors] = useState<EmissionFactor[]>([]);
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

  // The referential is profile-independent but needs an unlocked vault, so load
  // it once a profile (hence a session) is available.
  useEffect(() => {
    if (!profileId) {
      setEmissionFactors([]);
      return;
    }
    let cancelled = false;
    listEmissionFactorsUseCase()
      .then((factors) => {
        if (!cancelled) setEmissionFactors(factors);
      })
      .catch((e) => {
        if (!cancelled) handleError(e);
      });
    return () => {
      cancelled = true;
    };
  }, [profileId, handleError]);

  const reload = useCallback(async (): Promise<void> => {
    if (!profileId) {
      setCommutes([]);
      return;
    }
    setIsLoading(true);
    try {
      setCommutes(await listCommutesUseCase(profileId));
    } catch (e) {
      handleError(e);
    } finally {
      setIsLoading(false);
    }
  }, [profileId, handleError]);

  // Reload commutes whenever the active profile changes.
  useEffect(() => {
    if (!profileId) {
      setCommutes([]);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    listCommutesUseCase(profileId)
      .then((list) => {
        if (!cancelled) setCommutes(list);
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

  const createCommute = useCallback(
    async (input: CreateCommuteInput): Promise<boolean> => {
      if (!profileId) return false;
      setIsSubmitting(true);
      setError(null);
      try {
        await createCommuteUseCase(profileId, input);
        await reload(); // refresh so the indicative CO2 is recomputed
        return true;
      } catch (e) {
        handleError(e);
        return false;
      } finally {
        setIsSubmitting(false);
      }
    },
    [profileId, reload, handleError],
  );

  const updateCommute = useCallback(
    async (input: UpdateCommuteInput): Promise<boolean> => {
      setIsSubmitting(true);
      setError(null);
      try {
        await updateCommuteUseCase(input);
        await reload();
        return true;
      } catch (e) {
        handleError(e);
        return false;
      } finally {
        setIsSubmitting(false);
      }
    },
    [reload, handleError],
  );

  const deleteCommute = useCallback(
    async (id: string): Promise<boolean> => {
      setIsSubmitting(true);
      setError(null);
      try {
        await deleteCommuteUseCase(id);
        setCommutes((prev) => prev.filter((c) => c.id !== id));
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

  const clearError = useCallback(() => setError(null), []);

  const value = useMemo<CommuteContextValue>(
    () => ({
      commutes,
      emissionFactors,
      isLoading,
      isSubmitting,
      error,
      createCommute,
      updateCommute,
      deleteCommute,
      clearError,
    }),
    [
      commutes,
      emissionFactors,
      isLoading,
      isSubmitting,
      error,
      createCommute,
      updateCommute,
      deleteCommute,
      clearError,
    ],
  );

  return (
    <CommuteContext.Provider value={value}>{children}</CommuteContext.Provider>
  );
}
