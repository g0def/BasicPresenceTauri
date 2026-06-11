import { ChevronDown, ChevronUp, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import {
  MAX_MINUTES,
  MIN_MINUTES,
  STEP_MINUTES,
  type WorkEntry,
} from "@/features/work-hours/domain/entities/work-hours";
import {
  formatClockTime,
  formatMinutes,
} from "@/features/work-hours/presentation/duration-format";
import { cn } from "@/lib/utils";

interface WorkEntryListProps {
  entries: WorkEntry[];
  /** Day start (minutes since midnight); when set, each task shows its
   * computed clock range (entries stack from this time). */
  dayStartMinutes?: number | null;
  hoveredId: string | null;
  onHover: (id: string | null) => void;
  onMinutesChange: (id: string, minutes: number) => void;
  onMove: (id: string, direction: -1 | 1) => void;
  onRemove: (id: string) => void;
}

/** The day's tasks, first to last, with per-row duration slider and ordering
 * arrows. Hover is shared with the donut (row ↔ segment highlight). */
export function WorkEntryList({
  entries,
  dayStartMinutes,
  hoveredId,
  onHover,
  onMinutesChange,
  onMove,
  onRemove,
}: WorkEntryListProps) {
  const { t } = useTranslation();

  if (entries.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        {t("workHours.entries.empty")}
      </p>
    );
  }

  // Entries stack from the day start: entry N begins where N-1 ends.
  let cursor = dayStartMinutes ?? 0;
  const startTimes = entries.map((e) => {
    const startsAt = cursor;
    cursor += e.minutes;
    return startsAt;
  });

  return (
    <ol className="grid gap-2">
      {entries.map((entry, index) => (
        <li
          key={entry.id}
          onMouseEnter={() => onHover(entry.id)}
          onMouseLeave={() => onHover(null)}
          className={cn(
            "rounded-lg border border-l-4 bg-card p-3 transition-all duration-200",
            hoveredId === entry.id && "border-ring ring-1 ring-ring",
          )}
          style={{ borderLeftColor: entry.color }}
        >
          <div className="flex items-start gap-2">
            <span className="mt-0.5 w-5 shrink-0 text-sm font-semibold tabular-nums text-muted-foreground">
              {index + 1}.
            </span>
            <div className="min-w-0 flex-1">
              <div className="flex items-baseline gap-2">
                <span className="truncate font-medium">{entry.title}</span>
                <span className="ml-auto shrink-0 text-sm font-semibold tabular-nums">
                  {formatMinutes(entry.minutes)}
                </span>
              </div>
              {dayStartMinutes != null && (
                <p className="text-xs tabular-nums text-muted-foreground">
                  {formatClockTime(startTimes[index])} –{" "}
                  {formatClockTime(startTimes[index] + entry.minutes)}
                </p>
              )}
              {entry.description && (
                <p className="truncate text-xs text-muted-foreground">
                  {entry.description}
                </p>
              )}
            </div>
            <div className="flex shrink-0 items-center">
              <Button
                variant="ghost"
                size="icon-xs"
                disabled={index === 0}
                onClick={() => onMove(entry.id, -1)}
                aria-label={t("workHours.entries.moveUp")}
              >
                <ChevronUp />
              </Button>
              <Button
                variant="ghost"
                size="icon-xs"
                disabled={index === entries.length - 1}
                onClick={() => onMove(entry.id, 1)}
                aria-label={t("workHours.entries.moveDown")}
              >
                <ChevronDown />
              </Button>
              <Button
                variant="ghost"
                size="icon-xs"
                className="text-destructive"
                onClick={() => onRemove(entry.id)}
                aria-label={t("workHours.entries.remove")}
              >
                <Trash2 />
              </Button>
            </div>
          </div>
          <Slider
            className="mt-3"
            value={[entry.minutes]}
            min={MIN_MINUTES}
            max={MAX_MINUTES}
            step={STEP_MINUTES}
            onValueChange={([v]) => onMinutesChange(entry.id, v)}
            aria-label={t("workHours.form.duration")}
          />
        </li>
      ))}
    </ol>
  );
}
