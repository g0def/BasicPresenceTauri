import type { ProfileSettingsDto } from "@/features/profile-settings/data/dto/profile-settings.dto";
import {
  DEFAULT_PROFILE_SETTINGS,
  type ProfileSettings,
} from "@/features/profile-settings/domain/entities/profile-settings";
import type { CellDisplayMode } from "@/shared/cell-display/cell-display-mode";
import { NOTE_FONTS, type NoteFont } from "@/shared/note-font/note-font";

function coerceNoteFont(value: string): NoteFont {
  return NOTE_FONTS.includes(value as NoteFont) ? (value as NoteFont) : "sans";
}

function coerceCellMode(value: string): CellDisplayMode {
  return value === "hours" ? "hours" : "co2";
}

/** Defensive clamp: the backend already validates, but keep the UI in range —
 * and never let a missing/garbled value reach the picker as NaN. */
function clampStart(minutes: number): number {
  const n = Math.round(minutes);
  if (!Number.isFinite(n)) return DEFAULT_PROFILE_SETTINGS.startMinutes;
  return Math.min(1439, Math.max(0, n));
}

export function toProfileSettings(dto: ProfileSettingsDto): ProfileSettings {
  return {
    startMinutes: clampStart(dto.defaultStartMinutes),
    noteFont: coerceNoteFont(dto.noteFont),
    cellDisplayMode: coerceCellMode(dto.cellDisplayMode),
    co2: {
      gridCountry: dto.gridCountry,
      defaultCarOccupancy: dto.defaultCarOccupancy,
      includeRadiativeForcing: dto.includeRadiativeForcing,
      countBuildingEnergy: dto.countBuildingEnergy,
      workingDaysPerYear: dto.workingDaysPerYear,
      factorYear: dto.factorYear,
    },
  };
}

export function toProfileSettingsDto(s: ProfileSettings): ProfileSettingsDto {
  return {
    defaultStartMinutes: s.startMinutes,
    noteFont: s.noteFont,
    cellDisplayMode: s.cellDisplayMode,
    gridCountry: s.co2.gridCountry,
    defaultCarOccupancy: s.co2.defaultCarOccupancy,
    includeRadiativeForcing: s.co2.includeRadiativeForcing,
    countBuildingEnergy: s.co2.countBuildingEnergy,
    workingDaysPerYear: s.co2.workingDaysPerYear,
    factorYear: s.co2.factorYear,
  };
}
