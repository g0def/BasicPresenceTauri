import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import { formatCo2 } from "@/features/commute/presentation/commute-format";
import type {
  Granularity,
  PeriodBucket,
} from "@/features/stats/domain/aggregate";
import { EChart } from "@/features/stats/presentation/components/echart";
import type { ECOption } from "@/features/stats/presentation/echarts-setup";
import {
  localeFor,
  monthLabel,
  resolveCssColor,
  useThemeTick,
  weekLabel,
} from "@/features/stats/presentation/stats-format";
import { formatMinutes } from "@/features/work-hours/presentation/duration-format";

interface PeriodBarChartProps {
  buckets: PeriodBucket[];
  metric: "co2Kg" | "workMinutes";
  granularity: Granularity;
  /** Visible window as bucket indices (controlled by the page's range select);
   * the slider can still be dragged within the full timeline for a quick look. */
  startIndex: number;
  endIndex: number;
  /** CSS color expression for the bars, e.g. `"var(--success)"`. */
  color: string;
  ariaLabel: string;
}

/** A bar per period (month/week) for one metric, with an average reference line
 * that tracks the visible window. Reused for both CO₂ and worked-hours charts. */
export function PeriodBarChart({
  buckets,
  metric,
  granularity,
  startIndex,
  endIndex,
  color,
  ariaLabel,
}: PeriodBarChartProps) {
  const { t, i18n } = useTranslation();
  const themeTick = useThemeTick();
  const lang = i18n.resolvedLanguage ?? "fr";

  const option = useMemo<ECOption>(() => {
    const locale = localeFor(lang);
    const formatValue = (value: number) =>
      metric === "co2Kg" ? formatCo2(value, lang) : formatMinutes(value);
    // Empty periods (null) and ECharts' empty-value placeholder both read back as
    // null/NaN — show an em dash instead of "NaN" when hovering a gap.
    const formatMaybe = (v: unknown) =>
      v == null || !Number.isFinite(Number(v)) ? "—" : formatValue(Number(v));

    const labels = buckets.map((b) =>
      granularity === "month"
        ? monthLabel(b.refDay, locale)
        : weekLabel(b.refDay, t),
    );
    // Gap-filled periods (no days) become null: no phantom bar, and they're
    // excluded from the markLine average so it reflects only active periods.
    const values = buckets.map((b) => (b.dayCount > 0 ? b[metric] : null));

    const barColor = resolveCssColor(color);
    const axisColor = resolveCssColor("var(--muted-foreground)");
    const gridColor = resolveCssColor("var(--border)");
    const avgColor = resolveCssColor("var(--foreground)");

    // The page picks the window (date selectors); the slider stays draggable for
    // a quick look. filterMode "filter" drops out-of-window periods, so the Y
    // scale fits the window AND the "average" markLine reflects only the visible
    // periods.
    return {
      grid: { top: 24, right: 16, bottom: 64, left: 52 },
      dataZoom: [
        {
          type: "inside",
          startValue: startIndex,
          endValue: endIndex,
          filterMode: "filter",
        },
        {
          type: "slider",
          startValue: startIndex,
          endValue: endIndex,
          filterMode: "filter",
          bottom: 6,
          height: 18,
          borderColor: gridColor,
          backgroundColor: "transparent",
          fillerColor: "rgba(127, 127, 127, 0.18)",
          handleStyle: { color: barColor, borderColor: barColor },
          moveHandleStyle: { color: barColor },
          dataBackground: {
            lineStyle: { color: gridColor },
            areaStyle: { color: gridColor, opacity: 0.4 },
          },
          selectedDataBackground: {
            lineStyle: { color: barColor },
            areaStyle: { color: barColor, opacity: 0.25 },
          },
          textStyle: { color: axisColor },
          labelFormatter: (value) => labels[Math.round(Number(value))] ?? "",
        },
      ],
      tooltip: {
        trigger: "axis",
        valueFormatter: (v) => formatMaybe(v),
      },
      xAxis: {
        type: "category",
        data: labels,
        axisLabel: { color: axisColor, hideOverlap: true },
        axisLine: { lineStyle: { color: gridColor } },
        axisTick: { show: false },
      },
      yAxis: {
        type: "value",
        axisLabel: { color: axisColor, formatter: (v) => formatValue(Number(v)) },
        splitLine: { lineStyle: { color: gridColor } },
      },
      series: [
        {
          type: "bar",
          data: values,
          itemStyle: { color: barColor, borderRadius: [3, 3, 0, 0] },
          // ECharts averages the VISIBLE periods (filterMode "filter" removed
          // the rest) and moves this line live as the user zooms/pans.
          markLine: {
            symbol: "none",
            data: [{ type: "average" }],
            lineStyle: { color: avgColor, type: "dashed", opacity: 0.6 },
            label: {
              position: "insideEndTop",
              color: axisColor,
              // Hide the label when the window has no active periods (avg = NaN).
              formatter: (params) => {
                const value = Number(
                  (params as { value?: number | string }).value,
                );
                return Number.isFinite(value)
                  ? `${t("stats.average")} ${formatValue(value)}`
                  : "";
              },
            },
          },
        },
      ],
    };
    // themeTick is intentional: it isn't read here but bumps on light/dark
    // toggle so the memo recomputes and the canvas re-resolves its colors.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [buckets, metric, granularity, startIndex, endIndex, color, lang, t, themeTick]);

  return <EChart option={option} className="h-64 w-full" ariaLabel={ariaLabel} />;
}
