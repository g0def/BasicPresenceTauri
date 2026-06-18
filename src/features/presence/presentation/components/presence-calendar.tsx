import {
  addMonths,
  eachDayOfInterval,
  endOfMonth,
  endOfWeek,
  format,
  isSameMonth,
  isToday,
  isWeekend,
  startOfMonth,
  startOfWeek,
  subMonths,
} from "date-fns";
import { enUS, fr } from "date-fns/locale";
import { ChevronLeft, ChevronRight, Copy, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { formatCo2 } from "@/features/commute/presentation/commute-format";
import { PRESENCE_TYPES } from "@/features/presence/domain/entities/presence";
import { dayKey } from "@/features/presence/presentation/day-key";
import { PresenceDayDialog } from "@/features/presence/presentation/components/presence-day-dialog";
import { PRESENCE_STYLES } from "@/features/presence/presentation/components/presence-colors";
import { useDayClipboard } from "@/features/presence/presentation/hooks/use-day-clipboard";
import { usePresence } from "@/features/presence/presentation/hooks/use-presence";
import { formatMinutes } from "@/features/work-hours/presentation/duration-format";
import { useCellDisplayMode } from "@/shared/cell-display/use-cell-display-mode";
import { cn } from "@/lib/utils";

/** Minimalist month calendar: a grid of square day cells, colored per presence
 * type. Date arithmetic (month bounds, leap years, week alignment) is delegated
 * to date-fns; the component only renders and routes clicks to the dialog. */
export function PresenceCalendar() {
  const { t, i18n } = useTranslation();
  const {
    presencesByDay,
    currentMonth: month,
    setCurrentMonth: setMonth,
    reload,
  } = usePresence();

  const [selected, setSelected] = useState<Date | null>(null);
  const { mode: displayMode } = useCellDisplayMode();
  const { mode, arm, disarm, copyDay, pasteOnto } = useDayClipboard();

  // Reload presences on mount (ensures work hour edits are reflected immediately
  // when navigating back from the work hours page).
  useEffect(() => {
    void reload();
  }, [reload]);

  // Leave copy mode on Escape.
  useEffect(() => {
    if (mode === "idle") return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") disarm();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [mode, disarm]);

  const locale = i18n.resolvedLanguage === "en" ? enUS : fr;

  // The visible grid spans whole weeks (Monday-first), so it always renders a
  // clean rectangle — leading/trailing days fall in the adjacent months.
  const days = useMemo(() => {
    const start = startOfWeek(startOfMonth(month), { weekStartsOn: 1 });
    const end = endOfWeek(endOfMonth(month), { weekStartsOn: 1 });
    return eachDayOfInterval({ start, end });
  }, [month]);

  const weekdays = useMemo(
    () => days.slice(0, 7).map((d) => format(d, "EEEEEE", { locale })),
    [days, locale],
  );

  // Contextual label for the copy toggle (also its tooltip), driven by mode.
  const toggleLabel = t(
    mode === "pasting"
      ? "presence.copyMode.paste"
      : mode === "picking"
        ? "presence.copyMode.pick"
        : "presence.copyMode.start",
  );

  return (
    <section className="mt-6 flex flex-1 flex-col gap-3">
      <div className="relative flex items-center justify-center gap-4">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setMonth(subMonths(month, 1))}
          aria-label={t("presence.prevMonth")}
        >
          <ChevronLeft />
        </Button>
        <h2 className="min-w-48 text-center text-xl font-semibold capitalize">
          {format(month, "LLLL yyyy", { locale })}
        </h2>
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setMonth(addMonths(month, 1))}
          aria-label={t("presence.nextMonth")}
        >
          <ChevronRight />
        </Button>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              onClick={() => (mode === "idle" ? arm() : disarm())}
              aria-pressed={mode !== "idle"}
              aria-label={toggleLabel}
              className={cn(
                "absolute top-1/2 right-0 -translate-y-1/2",
                mode === "picking" && "text-orange-500 hover:text-orange-500",
                mode === "pasting" && "text-destructive hover:text-destructive",
              )}
            >
              <span className="relative inline-flex">
                <Copy />
                {mode === "pasting" && (
                  <X
                    className="absolute -top-1.5 -right-1.5 size-3 rounded-full bg-background"
                    aria-hidden
                  />
                )}
              </span>
            </Button>
          </TooltipTrigger>
          <TooltipContent className="max-w-xs">
            <p className="font-medium">{toggleLabel}</p>
            <p className="mt-1 text-muted-foreground">
              {t("presence.copyMode.help")}
            </p>
          </TooltipContent>
        </Tooltip>
      </div>

      <div className="grid grid-cols-7 gap-2">
        {weekdays.map((w, i) => (
          <div
            key={i}
            className="text-center text-xs font-medium text-muted-foreground capitalize"
          >
            {w}
          </div>
        ))}
      </div>

      <div className="grid flex-1 auto-rows-fr grid-cols-7 gap-2">
        {days.map((d) => {
          const key = dayKey(d);
          const presence = presencesByDay.get(key);
          const inMonth = isSameMonth(d, month);
          const today = isToday(d);
          const weekend = isWeekend(d);
          const style = presence ? PRESENCE_STYLES[presence.type] : null;
          const Icon = style?.Icon ?? null;
          // Weekends get a faint gray wash, unless a presence color overrides it.
          const emptyCell = weekend
            ? "bg-muted/50 hover:bg-accent"
            : "bg-card hover:bg-accent";
          const label = `${format(d, "PPP", { locale })}${
            style ? ` — ${t(style.labelKey)}` : ""
          }`;
          return (
            <button
              key={key}
              type="button"
              onClick={() => {
                if (mode === "picking") {
                  // Copy a fully-encoded day; empty days have nothing to copy.
                  if (presence) void copyDay(presence);
                } else if (mode === "pasting") {
                  // Paint only empty days; existing presences are left as is.
                  if (!presence) void pasteOnto(key);
                } else {
                  setSelected(d);
                }
              }}
              aria-label={label}
              className={cn(
                "flex flex-col items-start rounded-lg border p-2 text-sm transition-colors duration-300 ease-in-out",
                "hover:border-ring focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                inMonth
                  ? "border-border"
                  : "border-transparent text-muted-foreground/40",
                style ? style.cell : emptyCell,
                today && "ring-1 ring-ring",
                mode === "picking" &&
                  (presence ? "cursor-copy" : "cursor-not-allowed"),
                mode === "pasting" &&
                  (presence ? "cursor-not-allowed" : "cursor-copy"),
              )}
            >
              <span className="font-medium">{d.getDate()}</span>
              {presence &&
                (displayMode === "hours"
                  ? presence.workMinutes > 0 && (
                      <span className="m-auto text-sm font-semibold tabular-nums">
                        {formatMinutes(presence.workMinutes)}
                      </span>
                    )
                  : presence.co2Kg != null &&
                    presence.co2Kg > 0 && (
                      <span className="m-auto text-sm font-semibold tabular-nums">
                        {formatCo2(
                          presence.co2Kg,
                          i18n.resolvedLanguage ?? "fr",
                        )}
                        {presence.isEstimated ? " ~" : ""}
                      </span>
                    ))}
              {Icon && (
                <Icon
                  className="mt-auto size-5 self-end opacity-80"
                  aria-hidden
                />
              )}
            </button>
          );
        })}
      </div>

      <div className="flex flex-wrap justify-center gap-x-5 gap-y-2">
        {PRESENCE_TYPES.map((type) => {
          const Icon = PRESENCE_STYLES[type].Icon;
          return (
            <span
              key={type}
              className="flex items-center gap-1.5 text-xs text-muted-foreground"
            >
              <Icon className={cn("size-4", PRESENCE_STYLES[type].text)} />
              {t(PRESENCE_STYLES[type].labelKey)}
            </span>
          );
        })}
      </div>

      <PresenceDayDialog
        date={selected}
        open={selected !== null}
        onOpenChange={(o) => {
          if (!o) setSelected(null);
        }}
      />
    </section>
  );
}
