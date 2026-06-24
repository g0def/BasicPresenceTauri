import { Link } from "@tanstack/react-router";
import { ChevronLeft } from "lucide-react";
import { lazy, Suspense, useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import type { PresenceType } from "@/features/presence/domain/entities/presence";
import { PRESENCE_TYPES } from "@/features/presence/domain/entities/presence";
import { dayKey } from "@/features/presence/presentation/day-key";
import { usePresence } from "@/features/presence/presentation/hooks/use-presence";
import {
  breakdownByType,
  bucketPresences,
  periodKey,
  type Granularity,
} from "@/features/stats/domain/aggregate";
import { GranularityToggle } from "@/features/stats/presentation/components/granularity-toggle";
import { PeriodRangeSelect } from "@/features/stats/presentation/components/period-range-select";
import { PresenceTypeFilter } from "@/features/stats/presentation/components/presence-type-filter";
import {
  localeFor,
  monthLabel,
  weekLabel,
} from "@/features/stats/presentation/stats-format";

// Chart components pull in ECharts; lazy-load them so the library lands in its
// own chunk that only downloads when the stats page is opened.
const PeriodBarChart = lazy(() =>
  import(
    "@/features/stats/presentation/components/period-bar-chart"
  ).then((m) => ({ default: m.PeriodBarChart })),
);
const BreakdownPie = lazy(() =>
  import("@/features/stats/presentation/components/breakdown-pie").then((m) => ({
    default: m.BreakdownPie,
  })),
);

/** Periods the bar charts show by default (until the user narrows the range). */
const DEFAULT_VISIBLE = 12;

export function StatsPage() {
  const { t, i18n } = useTranslation();
  const { presencesByDay } = usePresence();
  const [granularity, setGranularity] = useState<Granularity>("month");
  const [enabled, setEnabled] = useState<Set<PresenceType>>(
    () => new Set(PRESENCE_TYPES),
  );
  // Both ranges are [startKey, endKey] bucket keys; null = the default window.
  // Reset whenever the period unit changes (keys aren't comparable across units).
  // `barRange` bounds the CO₂/hours charts; `breakdownRange` bounds the pie.
  const [barRange, setBarRange] = useState<[number, number] | null>(null);
  const [breakdownRange, setBreakdownRange] = useState<[number, number] | null>(
    null,
  );

  const locale = localeFor(i18n.resolvedLanguage ?? "fr");

  const presences = useMemo(
    () => Array.from(presencesByDay.values()),
    [presencesByDay],
  );

  // Full timeline up to today (so the current period is always shown, even with
  // no recent data) — the zoom slider picks the visible window and ECharts
  // averages it. `dayKey(new Date())` matches the UTC-midnight day convention.
  const buckets = useMemo(
    () => bucketPresences(presences, granularity, dayKey(new Date())),
    [presences, granularity],
  );

  // Selectable periods for the breakdown range, labelled in the active unit.
  const periods = useMemo(
    () =>
      buckets.map((b) => ({
        key: b.key,
        label:
          granularity === "month"
            ? monthLabel(b.refDay, locale)
            : weekLabel(b.refDay, t),
      })),
    [buckets, granularity, locale, t],
  );
  const fullStart = periods[0]?.key ?? 0;
  const fullEnd = periods[periods.length - 1]?.key ?? 0;
  const [rangeStart, rangeEnd] = breakdownRange ?? [fullStart, fullEnd];

  // Bar charts' visible window as bucket indices (what ECharts' dataZoom wants).
  // Defaults to the most recent DEFAULT_VISIBLE periods; the "De … à …" select
  // overrides it. findIndex falls back gracefully if a key ever goes stale.
  const lastIndex = Math.max(0, buckets.length - 1);
  const barStartIndex = barRange
    ? Math.max(
        0,
        buckets.findIndex((b) => b.key === barRange[0]),
      )
    : Math.max(0, buckets.length - DEFAULT_VISIBLE);
  const barEndIndex = barRange
    ? Math.max(
        barStartIndex,
        buckets.findIndex((b) => b.key === barRange[1]),
      )
    : lastIndex;
  const barStartKey = periods[barStartIndex]?.key ?? fullStart;
  const barEndKey = periods[barEndIndex]?.key ?? fullEnd;

  const slices = useMemo(() => {
    const inRange = presences.filter((p) => {
      const key = periodKey(p.day, granularity);
      return key >= rangeStart && key <= rangeEnd;
    });
    return breakdownByType(inRange, enabled);
  }, [presences, enabled, granularity, rangeStart, rangeEnd]);

  const toggle = useCallback((type: PresenceType) => {
    setEnabled((prev) => {
      const next = new Set(prev);
      if (next.has(type)) next.delete(type);
      else next.add(type);
      return next;
    });
  }, []);

  const changeGranularity = useCallback((next: Granularity) => {
    setGranularity(next);
    setBarRange(null);
    setBreakdownRange(null);
  }, []);

  return (
    <section className="mt-6 mx-auto flex w-full max-w-4xl flex-col gap-8">
      <div className="flex items-center gap-3">
        <Button variant="ghost" size="icon" asChild>
          <Link to="/" aria-label={t("stats.back")}>
            <ChevronLeft />
          </Link>
        </Button>
        <h2 className="text-xl font-semibold">{t("stats.title")}</h2>
      </div>

      {presences.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("stats.empty")}</p>
      ) : (
        <Suspense
          fallback={
            <p className="text-sm text-muted-foreground">
              {t("common.loading")}
            </p>
          }
        >
          <GranularityToggle value={granularity} onChange={changeGranularity} />

          <div className="flex flex-col gap-3">
            <h3 className="text-base font-medium">
              {t("stats.breakdownTitle")}
            </h3>
            {periods.length > 1 && (
              <PeriodRangeSelect
                periods={periods}
                start={rangeStart}
                end={rangeEnd}
                onChange={(start, end) => setBreakdownRange([start, end])}
              />
            )}
            <PresenceTypeFilter enabled={enabled} onToggle={toggle} />
            {slices.length > 0 ? (
              <BreakdownPie
                slices={slices}
                ariaLabel={t("stats.breakdownTitle")}
              />
            ) : (
              <p className="text-sm text-muted-foreground">{t("stats.empty")}</p>
            )}
          </div>

          {/* Separates the breakdown (above) from the period charts (below) so it
              reads at a glance which date range select drives which chart. */}
          <hr className="border-border" />

          <div className="flex flex-col gap-6">
            {periods.length > 1 && (
              <PeriodRangeSelect
                periods={periods}
                start={barStartKey}
                end={barEndKey}
                onChange={(start, end) => setBarRange([start, end])}
              />
            )}

            <div className="flex flex-col gap-3">
              <h3 className="text-base font-medium">{t("stats.co2Title")}</h3>
              <PeriodBarChart
                buckets={buckets}
                metric="co2Kg"
                granularity={granularity}
                startIndex={barStartIndex}
                endIndex={barEndIndex}
                color="var(--success)"
                ariaLabel={t("stats.co2Title")}
              />
            </div>

            <div className="flex flex-col gap-3">
              <h3 className="text-base font-medium">{t("stats.hoursTitle")}</h3>
              <PeriodBarChart
                buckets={buckets}
                metric="workMinutes"
                granularity={granularity}
                startIndex={barStartIndex}
                endIndex={barEndIndex}
                color="var(--primary)"
                ariaLabel={t("stats.hoursTitle")}
              />
            </div>
          </div>
        </Suspense>
      )}
    </section>
  );
}
