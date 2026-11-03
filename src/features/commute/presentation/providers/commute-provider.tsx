import { useCallback, useMemo } from "react";
import type { ReactNode } from "react";

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useContainer } from "@/core/di";
import { useCommand, useFeatureError } from "@/core/query";
import type {
  Commute,
  CreateCommuteInput,
  EmissionFactor,
  UpdateCommuteInput,
} from "@/entities/commute/model/commute";
import { makeCreateCommuteUseCase } from "@/features/commute/domain/use-cases/create-commute";
import { makeDeleteCommuteUseCase } from "@/features/commute/domain/use-cases/delete-commute";
import { makeListCommutesUseCase } from "@/features/commute/domain/use-cases/list-commutes";
import { makeListEmissionFactorsUseCase } from "@/features/commute/domain/use-cases/list-emission-factors";
import { makeUpdateCommuteUseCase } from "@/features/commute/domain/use-cases/update-commute";
import {
  CommuteContext,
  type CommuteContextValue,
} from "@/features/commute/presentation/context/commute-context";
import { useProfile } from "@/entities/profile/model/use-profile";

const NO_COMMUTES: Commute[] = [];
const NO_FACTORS: EmissionFactor[] = [];

const commuteKeys = {
  byProfile: (profileId: string | null) => ["commutes", profileId] as const,
  factors: ["emission-factors"] as const,
};

export function CommuteProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? null;

  const { commuteRepository, emissionFactorRepository } = useContainer();
  const useCases = useMemo(
    () => ({
      list: makeListCommutesUseCase(commuteRepository),
      create: makeCreateCommuteUseCase(commuteRepository),
      update: makeUpdateCommuteUseCase(commuteRepository),
      remove: makeDeleteCommuteUseCase(commuteRepository),
      factors: makeListEmissionFactorsUseCase(emissionFactorRepository),
    }),
    [commuteRepository, emissionFactorRepository],
  );
  const queryClient = useQueryClient();
  const { run, isPending: isSubmitting } = useCommand();
  // The referential is profile-independent but needs an unlocked vault, so it
  // loads once a profile (hence a session) is available.
  const factorsQuery = useQuery({
    queryKey: commuteKeys.factors,
    enabled: profileId !== null,
    queryFn: () => useCases.factors(),
  });

  // Reloads whenever the active profile changes (the key carries it).
  const commutesQuery = useQuery({
    queryKey: commuteKeys.byProfile(profileId),
    enabled: profileId !== null,
    queryFn: () => useCases.list(profileId as string),
  });

  const { error, reportError, clearError } = useFeatureError(
    t("errors.generic"),
    factorsQuery.error,
    commutesQuery.error,
  );

  // Refresh after a write so the indicative CO2 is recomputed by the backend.
  const reload = useCallback(async (): Promise<void> => {
    if (!profileId) return;
    await queryClient.invalidateQueries({
      queryKey: commuteKeys.byProfile(profileId),
    });
  }, [profileId, queryClient]);

  const mutate = useCallback(
    async (work: () => Promise<unknown>): Promise<boolean> => {
      clearError();
      try {
        await run(work);
        await reload();
        return true;
      } catch (e) {
        reportError(e);
        return false;
      }
    },
    [clearError, run, reload, reportError],
  );

  const createCommute = useCallback(
    async (input: CreateCommuteInput): Promise<boolean> => {
      if (!profileId) return false;
      return mutate(() => useCases.create(profileId, input));
    },
    [profileId, mutate, useCases],
  );

  const updateCommute = useCallback(
    (input: UpdateCommuteInput) => mutate(() => useCases.update(input)),
    [mutate, useCases],
  );

  const deleteCommute = useCallback(
    (id: string) => mutate(() => useCases.remove(id)),
    [mutate, useCases],
  );

  const commutes = (profileId && commutesQuery.data) || NO_COMMUTES;
  const emissionFactors = (profileId && factorsQuery.data) || NO_FACTORS;
  const isLoading =
    profileId !== null && (commutesQuery.isPending || commutesQuery.isFetching);

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
