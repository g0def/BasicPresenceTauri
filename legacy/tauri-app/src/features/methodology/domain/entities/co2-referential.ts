/** One per-country electric-grid override (e.g. `car_ev` per charging country). */
export interface GridVariant {
  country: string;
  value: number;
}

/** A full referential row, as shown on the methodology page (carries the
 * traceability metadata the trip picker omits: `value`, `scope`, `source`). */
export interface Co2ReferentialFactor {
  modeId: string;
  /** Backend (French) label; the UI prefers the i18n label, this is the fallback. */
  label: string;
  value: number;
  /** e.g. `"kgCO2e/veh.km"`. */
  unit: string;
  /** `car` | `active` | `public_transport` | `rail` | `air` | `building`. */
  category: string;
  isParam: boolean;
  scope: string | null;
  source: string | null;
  gridVariants: GridVariant[];
}

/** The whole CO2 referential for the active millésime, plus the radiative-forcing
 * multiplier in force for that year. */
export interface Co2Referential {
  factorYear: number;
  radiativeForcing: number;
  factors: Co2ReferentialFactor[];
}

/** Display order of categories on the methodology table. */
export const CATEGORY_ORDER = [
  "car",
  "active",
  "public_transport",
  "rail",
  "air",
  "building",
] as const;
