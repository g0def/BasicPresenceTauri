import type { CellDisplayMode } from "@/shared/cell-display/cell-display-mode";
import type { NoteFont } from "@/shared/note-font/note-font";

/** CO2 configuration, now stored per profile (mirrors the backend columns). */
export interface Co2Config {
  gridCountry: string;
  defaultCarOccupancy: number;
  includeRadiativeForcing: boolean;
  countBuildingEnergy: boolean;
  workingDaysPerYear: number;
  factorYear: number;
}

/** Typed per-profile settings, backed by the `profile_settings` table. */
export interface ProfileSettings {
  /** "Heure de départ par défaut" — minutes since midnight, 0..1439. */
  startMinutes: number;
  noteFont: NoteFont;
  cellDisplayMode: CellDisplayMode;
  co2: Co2Config;
}

/** A partial change merged onto the current settings before persisting. */
export type ProfileSettingsInput = Partial<Omit<ProfileSettings, "co2">> & {
  co2?: Partial<Co2Config>;
};

/**
 * Frontend mirror of the backend lazy default (`ProfileSettings::default()`).
 * Used as the pre-load fallback and when no profile is active. MUST stay aligned
 * with the Rust defaults / the `0010` column defaults.
 */
export const DEFAULT_PROFILE_SETTINGS: ProfileSettings = {
  startMinutes: 510, // 08:30
  noteFont: "sans",
  cellDisplayMode: "co2",
  co2: {
    gridCountry: "BE",
    defaultCarOccupancy: 1,
    includeRadiativeForcing: true,
    countBuildingEnergy: false,
    workingDaysPerYear: 220,
    factorYear: 2026,
  },
};
