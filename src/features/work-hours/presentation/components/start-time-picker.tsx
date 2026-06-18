import { useEffect, useState } from "react";

import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  formatClockTime,
  parseClockTime,
} from "@/features/work-hours/presentation/duration-format";

/** Arrow-key step (minutes); Shift steps a full hour. */
const STEP_MINUTES = 5;
/** Where arrow-stepping starts when no time is set yet. */
const DEFAULT_START_MINUTES = 8 * 60;

interface StartTimePickerProps {
  /** Current value in minutes since midnight, or `null` when unset. */
  value: number | null;
  /** Called with the new start time when a valid, changed value is committed. */
  onCommit: (startMinutes: number) => void;
  id?: string;
  label: string;
  hint?: string;
}

/**
 * A start-time picker as a single free-text field — WebKitGTK has no usable
 * `<input type="time">`. Typing is parsed leniently ("8", "830", "8h30"…),
 * ↑/↓ nudge by 5 min (Shift = 1 h), and the value commits on blur/Enter.
 * Shared by the work-hours day editor and the per-profile default in settings.
 */
export function StartTimePicker({
  value,
  onCommit,
  id = "start-time",
  label,
  hint,
}: StartTimePickerProps) {
  const [text, setText] = useState("");

  // Re-sync the field whenever the bound value changes.
  useEffect(() => {
    setText(value === null ? "" : formatClockTime(value));
  }, [value]);

  const commit = () => {
    const startMinutes = parseClockTime(text);
    if (startMinutes === null || value === startMinutes) {
      // Invalid or unchanged: snap the display back to the bound value.
      setText(value === null ? "" : formatClockTime(value));
      return;
    }
    onCommit(startMinutes);
  };

  const step = (direction: -1 | 1, wholeHour: boolean) => {
    const current = parseClockTime(text) ?? value ?? DEFAULT_START_MINUTES;
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
    <div className="grid gap-1">
      <Label htmlFor={id} className="text-xs text-muted-foreground">
        {label}
      </Label>
      <Input
        id={id}
        className="w-[5.5rem] text-center font-semibold tabular-nums"
        inputMode="numeric"
        autoComplete="off"
        placeholder="08:30"
        value={text}
        aria-invalid={invalid || undefined}
        title={hint}
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
  );
}
