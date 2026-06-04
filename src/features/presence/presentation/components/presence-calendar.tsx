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
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { PRESENCE_TYPES } from "@/features/presence/domain/entities/presence";
import { dayKey } from "@/features/presence/presentation/day-key";
import { PresenceDayDialog } from "@/features/presence/presentation/components/presence-day-dialog";
import { PRESENCE_STYLES } from "@/features/presence/presentation/components/presence-colors";
import { usePresence } from "@/features/presence/presentation/hooks/use-presence";
import { cn } from "@/lib/utils";

/** Minimalist month calendar: a grid of square day cells, colored per presence
 * type. Date arithmetic (month bounds, leap years, week alignment) is delegated
 * to date-fns; the component only renders and routes clicks to the dialog. */
export function PresenceCalendar() {
  const { t, i18n } = useTranslation();
  const { presencesByDay } = usePresence();

  const [month, setMonth] = useState(() => startOfMonth(new Date()));
  const [selected, setSelected] = useState<Date | null>(null);

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

  return (
    <section className="mt-6 flex flex-1 flex-col gap-3">
      <div className="flex items-center justify-center gap-4">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setMonth((m) => subMonths(m, 1))}
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
          onClick={() => setMonth((m) => addMonths(m, 1))}
          aria-label={t("presence.nextMonth")}
        >
          <ChevronRight />
        </Button>
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
              onClick={() => setSelected(d)}
              aria-label={label}
              className={cn(
                "flex flex-col items-start rounded-lg border p-2 text-sm transition-colors",
                "hover:border-ring focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none",
                inMonth
                  ? "border-border"
                  : "border-transparent text-muted-foreground/40",
                style ? style.cell : emptyCell,
                today && "ring-1 ring-ring",
              )}
            >
              <span className="font-medium">{d.getDate()}</span>
              {Icon && (
                <Icon className="mt-auto size-5 self-end opacity-80" aria-hidden />
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
