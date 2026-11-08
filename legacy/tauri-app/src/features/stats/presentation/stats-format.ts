import { format, getISOWeek } from "date-fns";
import type { Locale } from "date-fns";
import { enUS, fr } from "date-fns/locale";
import type { TFunction } from "i18next";
import { useEffect, useState } from "react";

export function localeFor(lang: string): Locale {
  return lang === "en" ? enUS : fr;
}

/**
 * Resolve a CSS color expression (e.g. `"var(--success)"`) to a plain `rgb()`
 * string for ECharts.
 *
 * Two hops are needed. (1) The Canvas renderer can't evaluate `var(--…)`, so we
 * read the variable back off a throwaway probe element the browser has already
 * resolved against the active (light/dark) theme. (2) The app's tokens are
 * authored in `oklch()`; ECharts/zrender's color parser does NOT understand
 * `oklch()`/`oklab()`, so when it derives a hover-emphasis shade or animates a
 * color it produces a broken (transparent) value and the bar/slice vanishes. We
 * rasterize the resolved color through a 1×1 canvas and read the sRGB bytes
 * back, handing ECharts an `rgb()` it can actually parse.
 */
export function resolveCssColor(expr: string): string {
  if (typeof document === "undefined") return expr;

  const probe = document.createElement("span");
  probe.style.color = expr;
  probe.style.position = "absolute";
  probe.style.opacity = "0";
  probe.style.pointerEvents = "none";
  document.body.appendChild(probe);
  const computed = getComputedStyle(probe).color;
  probe.remove();

  try {
    const canvas = document.createElement("canvas");
    canvas.width = 1;
    canvas.height = 1;
    const ctx = canvas.getContext("2d");
    if (!ctx) return computed || expr;
    // Seed with opaque black so an unparseable color degrades to black rather
    // than a transparent pixel (which reads back as an invisible chart element).
    ctx.fillStyle = "#000";
    ctx.fillStyle = computed || expr;
    ctx.fillRect(0, 0, 1, 1);
    const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data;
    return `rgb(${r}, ${g}, ${b})`;
  } catch {
    // Environments without a real canvas (e.g. jsdom) fall back to the resolved
    // computed value; the rasterization only matters in the actual WebView.
    return computed || expr;
  }
}

/** "janv. 26" / "Jan 26" — abbreviated standalone month + 2-digit year. */
export function monthLabel(refDay: number, locale: Locale): string {
  return format(new Date(refDay), "LLL yy", { locale });
}

/** "S12" / "W12" — the ISO week number, localized via the `stats.weekLabel` key. */
export function weekLabel(refDay: number, t: TFunction): string {
  return t("stats.weekLabel", { week: getISOWeek(new Date(refDay)) });
}

/**
 * Bumps a counter whenever the `.dark` class on `<html>` toggles. Canvas charts
 * can't track CSS variables on their own, so chart components include this tick
 * in their option memo to re-resolve colors and redraw on theme change.
 */
export function useThemeTick(): number {
  const [tick, setTick] = useState(0);
  useEffect(() => {
    const observer = new MutationObserver(() => setTick((n) => n + 1));
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });
    return () => observer.disconnect();
  }, []);
  return tick;
}
