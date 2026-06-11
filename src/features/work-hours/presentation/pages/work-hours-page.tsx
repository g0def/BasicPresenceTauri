import { useState } from "react";
import { Link } from "@tanstack/react-router";
import { format } from "date-fns";
import { enUS, fr } from "date-fns/locale";
import { ChevronLeft } from "lucide-react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import type { Presence } from "@/features/presence/domain/entities/presence";
import { PRESENCE_STYLES } from "@/features/presence/presentation/components/presence-colors";
import { DayDonut } from "@/features/work-hours/presentation/components/day-donut";
import { DayScheduleEditor } from "@/features/work-hours/presentation/components/day-schedule-editor";
import { TaskPresetPanel } from "@/features/work-hours/presentation/components/task-preset-panel";
import { WorkEntryList } from "@/features/work-hours/presentation/components/work-entry-list";
import { formatClockTime } from "@/features/work-hours/presentation/duration-format";
import { useWorkDay } from "@/features/work-hours/presentation/hooks/use-work-day";
import { cn } from "@/lib/utils";

interface WorkHoursPageProps {
  presence: Presence;
  date: Date;
}

/**
 * The day's hours-encoding page: saved/one-off tasks on the left, the day ring
 * in the center, the ordered task list (durations, ordering) on the right.
 * Edits persist live (debounced) — there is no save button.
 */
export function WorkHoursPage({ presence, date }: WorkHoursPageProps) {
  const { t, i18n } = useTranslation();
  const { logout } = useAuth();
  const {
    entries,
    totalMinutes,
    schedule,
    isLoading,
    isSaving,
    error,
    addEntry,
    updateMinutes,
    removeEntry,
    moveEntry,
    setSchedule,
  } = useWorkDay(presence.id, logout);
  const [hoveredId, setHoveredId] = useState<string | null>(null);

  const lang = i18n.resolvedLanguage ?? "fr";
  const locale = lang === "en" ? enUS : fr;
  const typeStyle = PRESENCE_STYLES[presence.type];

  return (
    <section className="mt-6 flex flex-1 flex-col gap-6">
      <div className="flex flex-wrap items-center gap-3">
        <Button variant="ghost" size="icon" asChild>
          <Link to="/" aria-label={t("workHours.back")}>
            <ChevronLeft />
          </Link>
        </Button>
        <h2 className="text-xl font-semibold capitalize">
          {format(date, "PPPP", { locale })}
        </h2>
        <span
          className={cn(
            "flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium",
            typeStyle.cell,
          )}
        >
          <typeStyle.Icon className="size-3" />
          {t(typeStyle.labelKey)}
        </span>
        <span
          className={cn(
            "ml-auto text-xs text-muted-foreground transition-opacity duration-300",
            isSaving ? "opacity-100" : "opacity-0",
          )}
          aria-live="polite"
        >
          {t("workHours.saving")}
        </span>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {isLoading ? (
        <p className="text-muted-foreground">{t("common.loading")}</p>
      ) : (
        <div className="grid items-start gap-8 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)_minmax(0,1fr)]">
          <TaskPresetPanel onAdd={addEntry} />

          <div className="flex flex-col items-center gap-4 lg:sticky lg:top-6">
            <DayDonut
              entries={entries}
              totalMinutes={totalMinutes}
              subtitle={
                schedule
                  ? `${formatClockTime(schedule.startMinutes)} – ${formatClockTime(schedule.endMinutes)}`
                  : null
              }
              hoveredId={hoveredId}
              onHover={setHoveredId}
            />
            <DayScheduleEditor schedule={schedule} onChange={setSchedule} />
            {entries.length === 0 && (
              <p className="max-w-60 text-center text-sm text-muted-foreground">
                {t("workHours.emptyHint")}
              </p>
            )}
          </div>

          <section className="flex flex-col gap-3">
            <h3 className="text-sm font-semibold">
              {t("workHours.entries.title")}
            </h3>
            <WorkEntryList
              entries={entries}
              dayStartMinutes={schedule?.startMinutes ?? null}
              hoveredId={hoveredId}
              onHover={setHoveredId}
              onMinutesChange={updateMinutes}
              onMove={moveEntry}
              onRemove={removeEntry}
            />
          </section>
        </div>
      )}
    </section>
  );
}
