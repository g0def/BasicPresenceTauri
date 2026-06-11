import { createContext } from "react";

import type {
  TaskPreset,
  TaskPresetInput,
} from "@/features/work-hours/domain/entities/work-hours";

export interface TaskPresetContextValue {
  /** Saved task presets of the active profile (alphabetical). */
  presets: TaskPreset[];
  isLoading: boolean;
  isSubmitting: boolean;
  error: string | null;
  /** Create a preset for the active profile. Returns `true` on success. */
  createPreset: (input: TaskPresetInput) => Promise<boolean>;
  /** Update a preset. Returns `true` on success. */
  updatePreset: (id: string, input: TaskPresetInput) => Promise<boolean>;
  /** Delete a preset by id. Returns `true` on success. */
  deletePreset: (id: string) => Promise<boolean>;
  clearError: () => void;
}

export const TaskPresetContext = createContext<TaskPresetContextValue | null>(
  null,
);
