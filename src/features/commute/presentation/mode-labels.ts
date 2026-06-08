import type { TFunction } from "i18next";

/** i18n keys for transport-mode labels, keyed by the backend `mode_id`. Kept in
 * sync with the seeded referential; `modeLabel` falls back to the backend-
 * provided label for any mode not listed here. */
export const MODE_LABEL_KEYS = {
  car_petrol: "commute.modes.car_petrol",
  car_diesel: "commute.modes.car_diesel",
  car_average: "commute.modes.car_average",
  car_phev: "commute.modes.car_phev",
  car_ev: "commute.modes.car_ev",
  taxi: "commute.modes.taxi",
  walk: "commute.modes.walk",
  bike: "commute.modes.bike",
  ebike: "commute.modes.ebike",
  escooter: "commute.modes.escooter",
  scooter_elec: "commute.modes.scooter_elec",
  public_transport: "commute.modes.public_transport",
  bus: "commute.modes.bus",
  coach: "commute.modes.coach",
  metro_tram: "commute.modes.metro_tram",
  train_sncb: "commute.modes.train_sncb",
  train_ter: "commute.modes.train_ter",
  train_hs_fr: "commute.modes.train_hs_fr",
  train_eurostar: "commute.modes.train_eurostar",
  train_thalys: "commute.modes.train_thalys",
  plane_domestic: "commute.modes.plane_domestic",
  plane_short: "commute.modes.plane_short",
  plane_medium: "commute.modes.plane_medium",
  plane_long: "commute.modes.plane_long",
} as const satisfies Record<string, string>;

/** Localized label for a transport mode; falls back to `fallback` (the backend
 * label) when the mode_id has no i18n entry. */
export function modeLabel(
  modeId: string,
  fallback: string,
  t: TFunction,
): string {
  if (modeId in MODE_LABEL_KEYS) {
    return t(MODE_LABEL_KEYS[modeId as keyof typeof MODE_LABEL_KEYS]);
  }
  return fallback;
}
