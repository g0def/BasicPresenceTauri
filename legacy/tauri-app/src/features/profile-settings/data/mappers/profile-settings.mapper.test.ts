import { describe, expect, it } from "vitest";

import {
  toProfileSettings,
  toProfileSettingsDto,
} from "@/features/profile-settings/data/mappers/profile-settings.mapper";
import type { ProfileSettingsDto } from "@/features/profile-settings/data/dto/profile-settings.dto";

const dto: ProfileSettingsDto = {
  defaultStartMinutes: 540,
  noteFont: "serif",
  cellDisplayMode: "hours",
  gridCountry: "FR",
  defaultCarOccupancy: 2,
  includeRadiativeForcing: false,
  countBuildingEnergy: true,
  workingDaysPerYear: 210,
  factorYear: 2025,
};

describe("profile-settings mapper", () => {
  it("maps the flat DTO into the nested domain shape", () => {
    const s = toProfileSettings(dto);
    expect(s.startMinutes).toBe(540);
    expect(s.noteFont).toBe("serif");
    expect(s.cellDisplayMode).toBe("hours");
    expect(s.co2).toEqual({
      gridCountry: "FR",
      defaultCarOccupancy: 2,
      includeRadiativeForcing: false,
      countBuildingEnergy: true,
      workingDaysPerYear: 210,
      factorYear: 2025,
    });
  });

  it("round-trips through the DTO", () => {
    expect(toProfileSettingsDto(toProfileSettings(dto))).toEqual(dto);
  });

  it("coerces unknown enums to safe defaults and clamps the start", () => {
    const s = toProfileSettings({
      ...dto,
      noteFont: "comic",
      cellDisplayMode: "pie",
      defaultStartMinutes: 5000,
    });
    expect(s.noteFont).toBe("sans");
    expect(s.cellDisplayMode).toBe("co2");
    expect(s.startMinutes).toBe(1439);
  });
});
