import { useCallback, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";

import { useContainer } from "@/core/di";
import { errorMessage, isSessionExpired, useCommand } from "@/core/query";
import type {
  TaskPreset,
  TaskPresetInput,
} from "@/entities/work-day/model/work-hours";
import { makeCreateTaskPresetUseCase } from "@/features/work-hours/domain/use-cases/create-task-preset";
import { makeDeleteTaskPresetUseCase } from "@/features/work-hours/domain/use-cases/delete-task-preset";
import { makeListTaskPresetsUseCase } from "@/features/work-hours/domain/use-cases/list-task-presets";
import { makeUpdateTaskPresetUseCase } from "@/features/work-hours/domain/use-cases/update-task-preset";
import {
  TaskPresetContext,
  type TaskPresetContextValue,
} from "@/features/work-hours/presentation/context/task-preset-context";
import { useProfile } from "@/entities/profile/model/use-profile";

const byTitle = (a: TaskPreset, b: TaskPreset) =>
  a.title.localeCompare(b.title);

const NO_PRESETS: TaskPreset[] = [];
const presetsKey = (profileId: string | null) =>
  ["task-presets", profileId] as const;

export function TaskPresetProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? null;

  const { taskPresetRepository } = useContainer();
  const useCases = useMemo(
    () => ({
      list: makeListTaskPresetsUseCase(taskPresetRepository),
      create: makeCreateTaskPresetUseCase(taskPresetRepository),
      update: makeUpdateTaskPresetUseCase(taskPresetRepository),
      remove: makeDeleteTaskPresetUseCase(taskPresetRepository),
    }),
    [taskPresetRepository],
  );
  const queryClient = useQueryClient();
  const { run, isPending: isSubmitting } = useCommand();
  const [error, setError] = useState<string | null>(null);

  const reportError = useCallback(
    (e: unknown) => {
      if (!isSessionExpired(e)) setError(errorMessage(e, t("errors.generic")));
    },
    [t],
  );

  // Reloads whenever the active profile changes (the key carries it).
  const query = useQuery({
    queryKey: presetsKey(profileId),
    enabled: profileId !== null,
    queryFn: () => useCases.list(profileId as string),
  });
  const presets = (profileId && query.data) || NO_PRESETS;
  const isLoading = profileId !== null && query.isPending;

  useEffect(() => {
    if (query.error) reportError(query.error);
  }, [query.error, reportError]);

  /** Apply a write, then patch the cached list with `patch`. */
  const mutate = useCallback(
    async <T,>(
      work: () => Promise<T>,
      patch: (prev: TaskPreset[], result: T) => TaskPreset[],
    ): Promise<boolean> => {
      setError(null);
      try {
        const result = await run(work);
        queryClient.setQueryData<TaskPreset[]>(
          presetsKey(profileId),
          (prev = []) => patch(prev, result),
        );
        return true;
      } catch (e) {
        reportError(e);
        return false;
      }
    },
    [profileId, run, queryClient, reportError],
  );

  const createPreset = useCallback(
    async (input: TaskPresetInput): Promise<boolean> => {
      if (!profileId) return false;
      return mutate(
        () => useCases.create(profileId, input),
        (prev, created) => [...prev, created].sort(byTitle),
      );
    },
    [profileId, mutate, useCases],
  );

  const updatePreset = useCallback(
    (id: string, input: TaskPresetInput) =>
      mutate(
        () => useCases.update(id, input),
        (prev, updated) =>
          prev.map((p) => (p.id === id ? updated : p)).sort(byTitle),
      ),
    [mutate, useCases],
  );

  const deletePreset = useCallback(
    (id: string) =>
      mutate(
        () => useCases.remove(id),
        (prev) => prev.filter((p) => p.id !== id),
      ),
    [mutate, useCases],
  );

  const clearError = useCallback(() => setError(null), []);

  const value = useMemo<TaskPresetContextValue>(
    () => ({
      presets,
      isLoading,
      isSubmitting,
      error,
      createPreset,
      updatePreset,
      deletePreset,
      clearError,
    }),
    [
      presets,
      isLoading,
      isSubmitting,
      error,
      createPreset,
      updatePreset,
      deletePreset,
      clearError,
    ],
  );

  return (
    <TaskPresetContext.Provider value={value}>
      {children}
    </TaskPresetContext.Provider>
  );
}
