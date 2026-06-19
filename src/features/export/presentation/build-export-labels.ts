import type { TFunction } from "i18next";

import type { ExportLabels } from "@/features/export/domain/entities/export";

/**
 * Build the spreadsheet labels from the active i18n language, so the produced
 * file is in the user's language. Presence-type labels reuse the existing
 * `presence.types.*` keys; the rest live under `export.sheet.*`.
 */
export function buildExportLabels(t: TFunction): ExportLabels {
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
    distance: t("export.sheet.cols.distance"),
    roundTrip: t("export.sheet.cols.roundTrip"),
    occupants: t("export.sheet.cols.occupants"),
    factorYear: t("export.sheet.cols.factorYear"),
    note: t("export.sheet.cols.note"),
  };
}
