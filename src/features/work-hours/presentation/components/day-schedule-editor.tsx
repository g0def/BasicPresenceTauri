import { useEffect, useState } from "react";
import { MoveRight } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { WorkDaySchedule } from "@/features/work-hours/domain/entities/work-hours";
import { formatClockTime } from "@/features/work-hours/presentation/duration-format";

interface DayScheduleEditorProps {
  schedule: WorkDaySchedule | null;
  onChange: (startMinutes: number) => void;
}

const HOURS = Array.from({ length: 24 }, (_, h) => String(h).padStart(2, "0"));
const MINUTES = Array.from({ length: 12 }, (_, i) =>
  String(i * 5).padStart(2, "0"),
);

/**
 * The day's start-time picker, as hour/minute selects in 24-hour format —
 * WebKitGTK has no usable `<input type="time">` (12-hour free text, no change
 * events). The end is never entered: the backend recomputes and stores it on
 * every save, and the stored value is what is displayed here.
 */
export function DayScheduleEditor({
  schedule,
  onChange,
}: DayScheduleEditorProps) {
  const { t } = useTranslation();

  const [hour, setHour] = useState("");
  const [minute, setMinute] = useState("");

  // Sync the selects whenever the loaded/saved schedule changes.
  useEffect(() => {
    if (schedule) {
      setHour(String(Math.floor(schedule.startMinutes / 60)).padStart(2, "0"));
      setMinute(String(schedule.startMinutes % 60).padStart(2, "0"));
    } else {
      setHour("");
      setMinute("");
    }
  }, [schedule]);

  const commit = (h: string, m: string) => {
    if (h === "" || m === "") return;
    const startMinutes = Number(h) * 60 + Number(m);
    if (schedule && schedule.startMinutes === startMinutes) return;
    onChange(startMinutes);
  };

  return (
    <div className="flex items-end gap-3">
      <div className="grid gap-1">
        <Label className="text-xs text-muted-foreground">
          {t("workHours.schedule.start")}
        </Label>
        <div className="flex items-center gap-1">
          <Select
            value={hour}
            onValueChange={(h) => {
              setHour(h);
              // Picking an hour with no minutes yet lands on a round hour.
              const m = minute === "" ? "00" : minute;
              setMinute(m);
              commit(h, m);
            }}
          >
            <SelectTrigger
              className="w-[4.5rem] tabular-nums"
              aria-label={t("workHours.schedule.start")}
            >
              <SelectValue placeholder="--" />
            </SelectTrigger>
            <SelectContent>
              {HOURS.map((h) => (
                <SelectItem key={h} value={h}>
                  {h}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <span className="text-sm font-semibold">:</span>
          <Select
            value={minute}
            onValueChange={(m) => {
              setMinute(m);
              commit(hour, m);
            }}
          >
            <SelectTrigger
              className="w-[4.5rem] tabular-nums"
              aria-label={t("workHours.schedule.start")}
            >
              <SelectValue placeholder="--" />
            </SelectTrigger>
            <SelectContent>
              {MINUTES.map((m) => (
                <SelectItem key={m} value={m}>
                  {m}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
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
