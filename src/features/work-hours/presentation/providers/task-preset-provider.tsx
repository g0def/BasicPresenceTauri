import { useCallback, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

import { isAppError } from "@/core/errors";
import { TauriTaskPresetRepository } from "@/features/work-hours/data/repositories/tauri-task-preset.repository";
import type {
  TaskPreset,
  TaskPresetInput,
} from "@/features/work-hours/domain/entities/work-hours";
import { makeCreateTaskPresetUseCase } from "@/features/work-hours/domain/use-cases/create-task-preset";
import { makeDeleteTaskPresetUseCase } from "@/features/work-hours/domain/use-cases/delete-task-preset";
import { makeListTaskPresetsUseCase } from "@/features/work-hours/domain/use-cases/list-task-presets";
import { makeUpdateTaskPresetUseCase } from "@/features/work-hours/domain/use-cases/update-task-preset";
import {
  TaskPresetContext,
  type TaskPresetContextValue,
} from "@/features/work-hours/presentation/context/task-preset-context";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";

// Composition root for the work-hours presets: wire repository to use cases.
const presetRepo = new TauriTaskPresetRepository();
const listPresetsUseCase = makeListTaskPresetsUseCase(presetRepo);
const createPresetUseCase = makeCreateTaskPresetUseCase(presetRepo);
const updatePresetUseCase = makeUpdateTaskPresetUseCase(presetRepo);
const deletePresetUseCase = makeDeleteTaskPresetUseCase(presetRepo);

const byTitle = (a: TaskPreset, b: TaskPreset) =>
  a.title.localeCompare(b.title);

interface TaskPresetProviderProps {
  children: ReactNode;
  /** Called when a command reports a locked vault (SESSION_EXPIRED). Injected
   * by the composition root (App) so the feature stays independent of auth. */
  onSessionExpired?: () => void;
}

export function TaskPresetProvider({
  children,
  onSessionExpired,
}: TaskPresetProviderProps) {
  const { activeProfile } = useProfile();
  const profileId = activeProfile?.id ?? null;

  const [presets, setPresets] = useState<TaskPreset[]>([]);
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

  // Reload presets whenever the active profile changes.
  useEffect(() => {
    if (!profileId) {
      setPresets([]);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    listPresetsUseCase(profileId)
      .then((list) => {
        if (!cancelled) setPresets(list);
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

  const createPreset = useCallback(
    async (input: TaskPresetInput): Promise<boolean> => {
      if (!profileId) return false;
      setIsSubmitting(true);
      setError(null);
      try {
        const created = await createPresetUseCase(profileId, input);
        setPresets((prev) => [...prev, created].sort(byTitle));
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

  const updatePreset = useCallback(
    async (id: string, input: TaskPresetInput): Promise<boolean> => {
      setIsSubmitting(true);
      setError(null);
      try {
        const updated = await updatePresetUseCase(id, input);
        setPresets((prev) =>
          prev.map((p) => (p.id === id ? updated : p)).sort(byTitle),
        );
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

  const deletePreset = useCallback(
    async (id: string): Promise<boolean> => {
      setIsSubmitting(true);
      setError(null);
      try {
        await deletePresetUseCase(id);
        setPresets((prev) => prev.filter((p) => p.id !== id));
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
