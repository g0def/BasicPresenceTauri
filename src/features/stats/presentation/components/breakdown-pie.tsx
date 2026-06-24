import { useMemo } from "react";
import { useTranslation } from "react-i18next";

import type { PresenceType } from "@/features/presence/domain/entities/presence";
import { PRESENCE_STYLES } from "@/features/presence/presentation/components/presence-colors";
import type { BreakdownSlice } from "@/features/stats/domain/aggregate";
import { EChart } from "@/features/stats/presentation/components/echart";
import type { ECOption } from "@/features/stats/presentation/echarts-setup";
import {
  resolveCssColor,
  useThemeTick,
} from "@/features/stats/presentation/stats-format";

/** Same semantic tokens as the calendar legend, so the pie matches the cells. */
const TYPE_COLOR_VAR: Record<PresenceType, string> = {
  office: "var(--primary)",
  remote: "var(--success)",
  vacation: "var(--warning)",
  holiday: "var(--destructive)",
};

interface BreakdownPieProps {
  slices: BreakdownSlice[];
  ariaLabel: string;
}

/** Donut of day-counts per presence type (already filtered by the parent). */
export function BreakdownPie({ slices, ariaLabel }: BreakdownPieProps) {
  const { t } = useTranslation();
  const themeTick = useThemeTick();

  const option = useMemo<ECOption>(() => {
    const textColor = resolveCssColor("var(--muted-foreground)");
    const data = slices.map((slice) => ({
      value: slice.days,
      name: t(PRESENCE_STYLES[slice.type].labelKey),
      itemStyle: { color: resolveCssColor(TYPE_COLOR_VAR[slice.type]) },
    }));

    return {
      tooltip: { trigger: "item", formatter: "{b}: {c} ({d}%)" },
      legend: { bottom: 0, textStyle: { color: textColor } },
      series: [
        {
          type: "pie",
          radius: ["45%", "70%"],
          center: ["50%", "44%"],
          data,
          label: { color: textColor },
        },
      ],
    };
    // themeTick is intentional: it isn't read here but bumps on light/dark
    // toggle so the memo recomputes and the canvas re-resolves its colors.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [slices, t, themeTick]);

  return <EChart option={option} className="h-72 w-full" ariaLabel={ariaLabel} />;
}
