import { useContext } from "react";

import {
  TaskPresetContext,
  type TaskPresetContextValue,
} from "@/features/work-hours/presentation/context/task-preset-context";

export function useTaskPresets(): TaskPresetContextValue {
  const ctx = useContext(TaskPresetContext);
  if (!ctx) {
    throw new Error("useTaskPresets must be used within <TaskPresetProvider>");
  }
  return ctx;
}
