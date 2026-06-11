import { useEffect, useState } from "react";
import { MoveRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { WorkDaySchedule } from "@/features/work-hours/domain/entities/work-hours";
import {
  formatClockTime,
  parseClockTime,
} from "@/features/work-hours/presentation/duration-format";

interface DayScheduleEditorProps {
  schedule: WorkDaySchedule | null;
  onChange: (startMinutes: number) => void;
}

/** Arrow-key step (minutes); Shift steps a full hour. */
const STEP_MINUTES = 5;
/** Where arrow-stepping starts when no time is set yet. */
const DEFAULT_START_MINUTES = 8 * 60;

/**
 * The day's start-time picker, as a single free-text field — WebKitGTK has no
 * usable `<input type="time">` (12-hour free text, no change events). Typing
 * is parsed leniently ("8", "830", "8h30"…), ↑/↓ nudge by 5 min (Shift = 1 h),
 * and the value commits on blur/Enter. The end is never entered: the backend
 * recomputes and stores it on every save, and the stored value is shown here.
 */
export function DayScheduleEditor({
  schedule,
  onChange,
}: DayScheduleEditorProps) {
  const { t } = useTranslation();

  const [text, setText] = useState("");

  // Re-sync the field whenever the loaded/saved schedule changes.
  useEffect(() => {
    setText(schedule ? formatClockTime(schedule.startMinutes) : "");
  }, [schedule]);

  const commit = () => {
    const startMinutes = parseClockTime(text);
    if (startMinutes === null || schedule?.startMinutes === startMinutes) {
      // Invalid or unchanged: snap the display back to the saved value.
      setText(schedule ? formatClockTime(schedule.startMinutes) : "");
      return;
    }
    onChange(startMinutes);
  };

  const step = (direction: -1 | 1, wholeHour: boolean) => {
    const current =
      parseClockTime(text) ?? schedule?.startMinutes ?? DEFAULT_START_MINUTES;
    const amount = wholeHour ? 60 : STEP_MINUTES;
    // Stepping from a value off the grid first snaps onto it (8:32 ↓ → 8:30).
    const snapped =
      direction === 1
        ? Math.floor(current / amount) * amount
        : Math.ceil(current / amount) * amount;
    const next = Math.min(1435, Math.max(0, snapped + direction * amount));
    setText(formatClockTime(next));
  };

  const invalid = text !== "" && parseClockTime(text) === null;

  return (
    <div className="flex items-end gap-3">
      <div className="grid gap-1">
        <Label
          htmlFor="day-start-time"
          className="text-xs text-muted-foreground"
        >
          {t("workHours.schedule.start")}
        </Label>
        <Input
          id="day-start-time"
          className="w-[5.5rem] text-center font-semibold tabular-nums"
          inputMode="numeric"
          autoComplete="off"
          placeholder="08:30"
          value={text}
          aria-invalid={invalid || undefined}
          title={t("workHours.schedule.startHint")}
          onChange={(e) => setText(e.target.value)}
          onFocus={(e) => e.target.select()}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              commit();
              e.currentTarget.blur();
            } else if (e.key === "ArrowUp" || e.key === "ArrowDown") {
              e.preventDefault();
              step(e.key === "ArrowUp" ? 1 : -1, e.shiftKey);
            }
          }}
        />
      </div>
      <MoveRight className="mb-2.5 size-4 text-muted-foreground" />
      <div className="grid gap-1">
        <span className="text-xs text-muted-foreground">
          {t("workHours.schedule.end")}
        </span>
        <span className="flex h-9 items-center text-sm font-semibold tabular-nums">
          {schedule ? formatClockTime(schedule.endMinutes) : "—"}
        </span>
      </div>
    </div>
  );
}
