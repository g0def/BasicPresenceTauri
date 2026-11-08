import { useEffect, useRef } from "react";

import {
  echarts,
  type ECOption,
} from "@/features/stats/presentation/echarts-setup";

interface EChartProps {
  option: ECOption;
  className?: string;
  ariaLabel?: string;
}

/**
 * Thin React wrapper around an ECharts instance: init once on mount, push new
 * options when they change, resize with the container, dispose on unmount.
 * Stateless by design — all data and (already-resolved) colors arrive through
 * `option`, in the spirit of the hand-rolled `DayDonut`.
 */
export function EChart({ option, className, ariaLabel }: EChartProps) {
  const elRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<ReturnType<typeof echarts.init> | null>(null);

  useEffect(() => {
    const el = elRef.current;
    if (!el) return;
    const chart = echarts.init(el, null, { renderer: "canvas" });
    chartRef.current = chart;
    const observer = new ResizeObserver(() => chart.resize());
    observer.observe(el);
    return () => {
      observer.disconnect();
      chart.dispose();
      chartRef.current = null;
    };
  }, []);

  useEffect(() => {
    // `notMerge: true` so toggling granularity/filters never leaves orphaned
    // series, bars or markLines from the previous option behind.
    chartRef.current?.setOption(option, true);
  }, [option]);

  return (
    <div ref={elRef} className={className} role="img" aria-label={ariaLabel} />
  );
}
