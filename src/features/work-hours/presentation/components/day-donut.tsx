import { useTranslation } from "react-i18next";

import type { WorkEntry } from "@/features/work-hours/domain/entities/work-hours";
import { computeDonutArcs } from "@/features/work-hours/presentation/donut-geometry";
import { formatMinutes } from "@/features/work-hours/presentation/duration-format";
import { cn } from "@/lib/utils";

interface DayDonutProps {
  entries: WorkEntry[];
  totalMinutes: number;
  /** Extra line under the total (e.g. the day's "08:30 – 17:00" range). */
  subtitle?: string | null;
  /** Entry highlighted from either the ring or the list (shared hover state). */
  hoveredId: string | null;
  onHover: (id: string | null) => void;
}

const RADIUS = 80;
const STROKE = 18;
const STROKE_HOVERED = 24;

/**
 * The day ring: total hours in the center, one colored arc per task around it,
 * proportional to duration. Hand-rolled SVG — each segment is a `<circle>`
 * normalized to `pathLength` 100 and placed via stroke-dasharray/offset, so
 * duration changes animate with plain CSS transitions and the colors/theme
 * need no chart-library glue.
 */
export function DayDonut({
  entries,
  totalMinutes,
  subtitle,
  hoveredId,
  onHover,
}: DayDonutProps) {
  const { t } = useTranslation();
  const arcs = computeDonutArcs(entries);
  const hovered = hoveredId
    ? (entries.find((e) => e.id === hoveredId) ?? null)
    : null;

  return (
    <div className="relative mx-auto w-full max-w-md select-none">
      <svg viewBox="0 0 200 200" role="img" aria-label={t("workHours.title")}>
        <g transform="rotate(-90 100 100)">
          {/* Background track, visible while the ring is empty or partial. */}
          <circle
            cx="100"
            cy="100"
            r={RADIUS}
            fill="none"
            strokeWidth={STROKE}
            className="stroke-muted"
          />
          {arcs.map((arc) => {
            const entry = entries.find((e) => e.id === arc.id);
            if (!entry) return null;
            const isHovered = hoveredId === entry.id;
            const isDimmed = hoveredId !== null && !isHovered;
            return (
              <circle
                key={entry.id}
                cx="100"
                cy="100"
                r={RADIUS}
                fill="none"
                pathLength={100}
                stroke={entry.color}
                strokeWidth={isHovered ? STROKE_HOVERED : STROKE}
                strokeDasharray={`${arc.length} ${100 - arc.length}`}
                strokeDashoffset={-arc.start}
                className={cn(
                  "cursor-pointer transition-all duration-300 ease-out",
                  isDimmed && "opacity-40",
                )}
                onMouseEnter={() => onHover(entry.id)}
                onMouseLeave={() => onHover(null)}
              >
                <title>{`${entry.title} — ${formatMinutes(entry.minutes)}`}</title>
              </circle>
            );
          })}
        </g>
      </svg>

      {/* Center label (HTML overlay so it uses the app's typography/tokens). */}
      <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center gap-1 px-12 text-center">
        {hovered ? (
          <>
            <span className="line-clamp-2 text-sm font-medium">
              {hovered.title}
            </span>
            <span
              className="text-3xl font-semibold tabular-nums"
              style={{ color: hovered.color }}
            >
              {formatMinutes(hovered.minutes)}
            </span>
          </>
        ) : (
          <>
            <span className="text-4xl font-semibold tabular-nums">
              {formatMinutes(totalMinutes)}
            </span>
            <span className="text-sm text-muted-foreground">
              {t("workHours.totalLabel")}
            </span>
            {subtitle && (
              <span className="text-xs tabular-nums text-muted-foreground">
                {subtitle}
              </span>
            )}
          </>
        )}
      </div>
    </div>
  );
}
