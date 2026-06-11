import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import {
  MAX_MINUTES,
  MIN_MINUTES,
  STEP_MINUTES,
  type TaskPreset,
  type TaskPresetInput,
} from "@/features/work-hours/domain/entities/work-hours";
import { formatMinutes } from "@/features/work-hours/presentation/duration-format";
import {
  DEFAULT_TASK_COLOR,
  TASK_COLOR_PALETTE,
} from "@/features/work-hours/presentation/work-hours-palette";
import { cn } from "@/lib/utils";

/** `custom` adds a one-off task to the day without saving a preset. */
export type TaskFormMode = "preset-create" | "preset-edit" | "custom";

interface TaskPresetFormDialogProps {
  open: boolean;
  mode: TaskFormMode;
  /** The preset being edited (mode `preset-edit`). */
  preset?: TaskPreset | null;
  isSubmitting?: boolean;
  onOpenChange: (open: boolean) => void;
  /** Receives the validated fields; the caller decides save-vs-add semantics. */
  onSubmit: (input: TaskPresetInput) => void;
}

const TITLE_KEYS = {
  "preset-create": "workHours.presets.add",
  "preset-edit": "workHours.presets.edit",
  custom: "workHours.custom.add",
} as const;

/** Shared form for creating/editing a preset and for one-off (custom) tasks:
 * title, description, 5-min-step duration slider and color choice. */
export function TaskPresetFormDialog({
  open,
  mode,
  preset,
  isSubmitting = false,
  onOpenChange,
  onSubmit,
}: TaskPresetFormDialogProps) {
  const { t } = useTranslation();

  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [minutes, setMinutes] = useState(60);
  const [color, setColor] = useState<string>(DEFAULT_TASK_COLOR);

  // Reset the form whenever the dialog (re)opens or targets another preset.
  useEffect(() => {
    if (!open) return;
    setTitle(preset?.title ?? "");
    setDescription(preset?.description ?? "");
    setMinutes(preset?.defaultMinutes ?? 60);
    setColor(preset?.color ?? DEFAULT_TASK_COLOR);
  }, [open, preset]);

  const canSubmit = title.trim().length > 0 && !isSubmitting;

  const submit = () => {
    if (!canSubmit) return;
    const trimmedDescription = description.trim();
    onSubmit({
      title: title.trim(),
      description: trimmedDescription || undefined,
      defaultMinutes: minutes,
      color,
    });
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{t(TITLE_KEYS[mode])}</DialogTitle>
          <DialogDescription>
            {mode === "custom"
              ? t("workHours.custom.hint")
              : t("workHours.presets.hint")}
          </DialogDescription>
        </DialogHeader>

        <form
          className="grid gap-4"
          onSubmit={(e) => {
            e.preventDefault();
            submit();
          }}
        >
          <div className="grid gap-1.5">
            <Label htmlFor="task-title">{t("workHours.form.title")}</Label>
            <Input
              id="task-title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              disabled={isSubmitting}
              autoFocus
            />
          </div>

          <div className="grid gap-1.5">
            <Label htmlFor="task-description">
              {t("workHours.form.description")}
            </Label>
            <Input
              id="task-description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              disabled={isSubmitting}
            />
          </div>

          <div className="grid gap-1.5">
            <div className="flex items-baseline justify-between">
              <Label>{t("workHours.form.duration")}</Label>
              <span className="text-sm font-semibold tabular-nums">
                {formatMinutes(minutes)}
              </span>
            </div>
            <Slider
              value={[minutes]}
              min={MIN_MINUTES}
              max={MAX_MINUTES}
              step={STEP_MINUTES}
              onValueChange={([v]) => setMinutes(v)}
              disabled={isSubmitting}
              aria-label={t("workHours.form.duration")}
            />
          </div>

          <div className="grid gap-1.5">
            <Label>{t("workHours.form.color")}</Label>
            <div className="flex flex-wrap items-center gap-2">
              {TASK_COLOR_PALETTE.map((swatch) => (
                <button
                  key={swatch}
                  type="button"
                  disabled={isSubmitting}
                  onClick={() => setColor(swatch)}
                  className={cn(
                    "size-7 rounded-full border transition-transform hover:scale-110",
                    "focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                    color.toLowerCase() === swatch.toLowerCase() &&
                      "ring-2 ring-ring ring-offset-2 ring-offset-background",
                  )}
                  style={{ backgroundColor: swatch }}
                  aria-label={swatch}
                />
              ))}
              <input
                type="color"
                value={color}
                disabled={isSubmitting}
                onChange={(e) => setColor(e.target.value)}
                className="size-7 cursor-pointer rounded-full border bg-transparent p-0.5"
                aria-label={t("workHours.form.customColor")}
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isSubmitting}
            >
              {t("workHours.form.cancel")}
            </Button>
            <Button type="submit" disabled={!canSubmit}>
              {mode === "custom"
                ? t("workHours.form.add")
                : t("workHours.form.save")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
