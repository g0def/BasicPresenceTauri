import { useState } from "react";

import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  formatClockTime,
  parseClockTime,
} from "@/shared/format/duration-format";

/** Arrow-key step (minutes); Shift steps a full hour. */
const STEP_MINUTES = 5;
/** Where arrow-stepping starts when no time is set yet. */
const DEFAULT_START_MINUTES = 8 * 60;

interface TimePickerProps {
  /** Current value in minutes since midnight, or `null` when unset. */
  value: number | null;
  /** Called with the new time when a valid, changed value is committed. */
  onCommit: (minutes: number) => void;
  id?: string;
  label: string;
  hint?: string;
  /** Flags the field invalid for a rule checked by the caller (e.g. end
   *  before start), on top of unparsable text. */
  invalid?: boolean;
}

/**
 * A 24-hour clock-time picker as a single free-text field — WebKitGTK has no
 * usable `<input type="time">` (it follows the OS locale, e.g. "5:00 PM").
 * Typing is parsed leniently ("8", "830", "8h30"…), ↑/↓ nudge by 5 min
 * (Shift = 1 h), and the value commits on blur/Enter. Shared by the
 * work-hours day editor, the per-profile default start in settings and the
 * Odoo export's fallback hours.
 */
export function TimePicker({
  value,
  onCommit,
  id = "start-time",
  label,
  hint,
  invalid: invalidByCaller = false,
}: TimePickerProps) {
  const [text, setText] = useState(() =>
    value === null ? "" : formatClockTime(value),
  );

  // Re-sync the field whenever the bound value changes (adjusted during
  // render, not in an effect).
  const [syncedWith, setSyncedWith] = useState(value);
  if (value !== syncedWith) {
    setSyncedWith(value);
    setText(value === null ? "" : formatClockTime(value));
  }

  const commit = () => {
    const minutes = parseClockTime(text);
    if (minutes === null || value === minutes) {
      // Invalid or unchanged: snap the display back to the bound value.
      setText(value === null ? "" : formatClockTime(value));
      return;
    }
    onCommit(minutes);
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

  const invalid =
    invalidByCaller || (text !== "" && parseClockTime(text) === null);

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
