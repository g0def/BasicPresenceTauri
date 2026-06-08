import type { TFunction } from "i18next";

import type {
  CommuteSegment,
  EmissionFactor,
} from "@/features/commute/domain/entities/commute";
import { modeLabel } from "@/features/commute/presentation/mode-labels";

/** Format a footprint in kgCO2e, respecting the locale's decimal separator
 * (e.g. "4,8 kg" in fr, "4.8 kg" in en). */
export function formatCo2(kg: number, lang: string): string {
  const locale = lang === "en" ? "en-US" : "fr-FR";
  const value = new Intl.NumberFormat(locale, {
    maximumFractionDigits: 1,
  }).format(kg);
  return `${value} kg`;
}

/** Build a short, localized summary of a commute's segments, e.g.
 * "30 km Intercity train (SNCB) + 5 km Bicycle". */
export function summarizeSegments(
  segments: CommuteSegment[],
  factors: EmissionFactor[],
  t: TFunction,
): string {
  const labelById = new Map(factors.map((f) => [f.modeId, f.label]));
  return segments
    .map(
      (s) =>
        `${s.distanceKm} km ${modeLabel(s.modeId, labelById.get(s.modeId) ?? s.modeId, t)}`,
    )
    .join(" + ");
}
