import type { TFunction } from "i18next";

import { MODE_LABEL_KEYS } from "@/features/commute/presentation/mode-labels";
import type { ExportLabels } from "@/features/export/domain/entities/export";

/**
 * Decimal mark for `lang`, so the backend can render numbers *inside* the recap
 * text cell of the Présences sheet. Numeric cells are written as real numbers
 * and formatted by the reader's own spreadsheet, so only this prose fragment
 * needs a fixed rendering.
 *
 * Derived from Intl rather than exposed as an i18n key: a one-character
 * translation is invisible in review and a stray edit would silently corrupt
 * every number in every exported summary. Mirrors the locale mapping of
 * `formatCo2`. Note that the *group* separator is deliberately not sent — long
 * distances render ungrouped ("12000 km") in both languages.
 */
function decimalSeparatorFor(lang: string): string {
  const locale = lang === "en" ? "en-US" : "fr-FR";
  return (
    new Intl.NumberFormat(locale)
      .formatToParts(1.1)
      .find((part) => part.type === "decimal")?.value ?? "."
  );
}

/**
 * Build the spreadsheet labels from the active i18n language, so the produced
 * file is in the user's language. Presence-type labels reuse the existing
 * `presence.types.*` keys, transport-mode names reuse `commute.modes.*` via
 * {@link MODE_LABEL_KEYS}, and the rest live under `export.sheet.*`.
 */
export function buildExportLabels(t: TFunction, lang: string): ExportLabels {
  return {
    sheetPresences: t("export.sheet.presences"),
    sheetTasks: t("export.sheet.tasks"),
    sheetTrips: t("export.sheet.trips"),
    sheetNotes: t("export.sheet.notes"),
    date: t("export.sheet.cols.date"),
    yes: t("export.sheet.yes"),
    no: t("export.sheet.no"),
    co2: t("export.sheet.cols.co2"),
    presenceType: t("export.sheet.cols.type"),
    estimated: t("export.sheet.cols.estimated"),
    tripSummary: t("export.sheet.cols.tripSummary"),
    hours: t("export.sheet.cols.hours"),
    created: t("export.sheet.cols.created"),
    updated: t("export.sheet.cols.updated"),
    typeOffice: t("presence.types.office"),
    typeRemote: t("presence.types.remote"),
    typeVacation: t("presence.types.vacation"),
    typeHoliday: t("presence.types.holiday"),
    title: t("export.sheet.cols.title"),
    description: t("export.sheet.cols.description"),
    minutes: t("export.sheet.cols.minutes"),
    color: t("export.sheet.cols.color"),
    order: t("export.sheet.cols.order"),
    mode: t("export.sheet.cols.mode"),
    modeLabel: t("export.sheet.cols.modeLabel"),
    distanceOneWay: t("export.sheet.cols.distanceOneWay"),
    roundTrip: t("export.sheet.cols.roundTrip"),
    distanceCounted: t("export.sheet.cols.distanceCounted"),
    occupants: t("export.sheet.cols.occupants"),
    factorYear: t("export.sheet.cols.factorYear"),
    tripJoin: t("export.sheet.trip.join"),
    unitKm: t("export.sheet.trip.unitKm"),
    roundTripSuffix: t("export.sheet.trip.roundTripSuffix"),
    oneWaySuffix: t("export.sheet.trip.oneWaySuffix"),
    decimalSeparator: decimalSeparatorFor(lang),
    note: t("export.sheet.cols.note"),
    modeNames: Object.fromEntries(
      Object.entries(MODE_LABEL_KEYS).map(([modeId, key]) => [modeId, t(key)]),
    ),
  };
}
