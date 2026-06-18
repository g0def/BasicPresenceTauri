import { MoveRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { WorkDaySchedule } from "@/features/work-hours/domain/entities/work-hours";
import { StartTimePicker } from "@/features/work-hours/presentation/components/start-time-picker";
import { formatClockTime } from "@/features/work-hours/presentation/duration-format";

interface DayScheduleEditorProps {
  schedule: WorkDaySchedule | null;
  onChange: (startMinutes: number) => void;
}

/**
 * The day's start-time picker plus the derived end time. The start is edited
 * via the shared {@link StartTimePicker}; the end is never entered — the backend
 * recomputes and stores it on every save, and the stored value is shown here.
 */
export function DayScheduleEditor({
  schedule,
  onChange,
}: DayScheduleEditorProps) {
  const { t } = useTranslation();

  return (
    <div className="flex items-end gap-3">
      <StartTimePicker
        id="day-start-time"
        value={schedule?.startMinutes ?? null}
        onCommit={onChange}
        label={t("workHours.schedule.start")}
        hint={t("workHours.schedule.startHint")}
      />
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
