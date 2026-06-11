import { useState } from "react";
import { ListPlus, Pencil, Plus, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import type {
  TaskPreset,
  TaskPresetInput,
  WorkEntryInput,
} from "@/features/work-hours/domain/entities/work-hours";
import { useTaskPresets } from "@/features/work-hours/presentation/hooks/use-task-presets";
import {
  TaskPresetFormDialog,
  type TaskFormMode,
} from "@/features/work-hours/presentation/components/task-preset-form-dialog";
import { formatMinutes } from "@/features/work-hours/presentation/duration-format";

interface TaskPresetPanelProps {
  /** Adds a task to the day (preset click or validated one-off form). */
  onAdd: (input: WorkEntryInput) => void;
}

/** Left panel: the profile's saved task presets (click = add to the day) plus
 * a one-off task entry that is not kept for next time. */
export function TaskPresetPanel({ onAdd }: TaskPresetPanelProps) {
  const { t } = useTranslation();
  const {
    presets,
    isLoading,
    isSubmitting,
    error,
    createPreset,
    updatePreset,
    deletePreset,
  } = useTaskPresets();

  const [formMode, setFormMode] = useState<TaskFormMode | null>(null);
  const [editing, setEditing] = useState<TaskPreset | null>(null);

  const closeForm = () => {
    setFormMode(null);
    setEditing(null);
  };

  const onSubmit = async (input: TaskPresetInput) => {
    if (formMode === "custom") {
      onAdd({
        title: input.title,
        description: input.description,
        minutes: input.defaultMinutes,
        color: input.color,
      });
      closeForm();
      return;
    }
    const ok =
      formMode === "preset-edit" && editing
        ? await updatePreset(editing.id, input)
        : await createPreset(input);
    if (ok) closeForm();
  };

  return (
    <section className="flex flex-col gap-3">
      <div className="flex items-center justify-between gap-2">
        <h3 className="text-sm font-semibold">
          {t("workHours.presets.title")}
        </h3>
        <Button
          variant="outline"
          size="xs"
          onClick={() => setFormMode("preset-create")}
        >
          <Plus />
          {t("workHours.presets.add")}
        </Button>
      </div>

      {isLoading ? (
        <p className="text-sm text-muted-foreground">{t("common.loading")}</p>
      ) : presets.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          {t("workHours.presets.empty")}
        </p>
      ) : (
        <ul className="grid gap-2">
          {presets.map((preset) => (
            <li key={preset.id} className="group relative">
              <button
                type="button"
                onClick={() =>
                  onAdd({
                    title: preset.title,
                    description: preset.description ?? undefined,
                    minutes: preset.defaultMinutes,
                    color: preset.color,
                  })
                }
                className="w-full rounded-lg border bg-card p-3 text-left transition-colors hover:border-ring focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
              >
                <span className="flex items-center gap-2">
                  <span
                    className="size-3 shrink-0 rounded-full"
                    style={{ backgroundColor: preset.color }}
                    aria-hidden
                  />
                  <span className="truncate text-sm font-medium">
                    {preset.title}
                  </span>
                  <span className="ml-auto shrink-0 pr-12 text-xs font-semibold tabular-nums text-muted-foreground">
                    {formatMinutes(preset.defaultMinutes)}
                  </span>
                </span>
                {preset.description && (
                  <span className="mt-0.5 block truncate pl-5 text-xs text-muted-foreground">
                    {preset.description}
                  </span>
                )}
              </button>
              <span className="absolute top-2.5 right-2 flex opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100">
                <Button
                  variant="ghost"
                  size="icon-xs"
                  onClick={() => {
                    setEditing(preset);
                    setFormMode("preset-edit");
                  }}
                  aria-label={t("workHours.presets.edit")}
                >
                  <Pencil />
                </Button>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="text-destructive"
                  onClick={() => void deletePreset(preset.id)}
                  aria-label={t("workHours.presets.delete")}
                >
                  <Trash2 />
                </Button>
              </span>
            </li>
          ))}
        </ul>
      )}

      {error && <p className="text-sm text-destructive">{error}</p>}

      <Button variant="secondary" onClick={() => setFormMode("custom")}>
        <ListPlus />
        {t("workHours.custom.add")}
      </Button>

      <TaskPresetFormDialog
        open={formMode !== null}
        mode={formMode ?? "preset-create"}
        preset={formMode === "preset-edit" ? editing : null}
        isSubmitting={isSubmitting}
        onOpenChange={(o) => {
          if (!o) closeForm();
        }}
        onSubmit={(input) => void onSubmit(input)}
      />
    </section>
  );
}
