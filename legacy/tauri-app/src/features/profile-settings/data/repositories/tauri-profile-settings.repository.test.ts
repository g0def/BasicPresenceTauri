import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it } from "vitest";

import { TauriProfileSettingsRepository } from "@/features/profile-settings/data/repositories/tauri-profile-settings.repository";
import type { ProfileSettings } from "@/features/profile-settings/domain/entities/profile-settings";

const backendDto = {
  defaultStartMinutes: 540,
  noteFont: "mono",
  cellDisplayMode: "hours",
  gridCountry: "FR",
  defaultCarOccupancy: 2,
  includeRadiativeForcing: false,
  countBuildingEnergy: false,
  workingDaysPerYear: 220,
  factorYear: 2025,
};

describe("TauriProfileSettingsRepository", () => {
  afterEach(() => clearMocks());

  it("loads and maps the flat DTO to the nested domain shape", async () => {
    mockIPC((cmd, args) => {
      if (cmd === "get_profile_settings") {
        expect(args).toEqual({ profileId: "p1" });
        return backendDto;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriProfileSettingsRepository();
    const s = await repo.get("p1");
    expect(s.noteFont).toBe("mono");
    expect(s.co2.gridCountry).toBe("FR");
    expect(s.co2.defaultCarOccupancy).toBe(2);
  });

  it("sends the flat DTO under a `settings` arg on save", async () => {
    let received: unknown;
    mockIPC((cmd, args) => {
      if (cmd === "set_profile_settings") {
        received = args;
        return backendDto;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const settings: ProfileSettings = {
      startMinutes: 540,
      noteFont: "mono",
      cellDisplayMode: "hours",
      co2: {
        gridCountry: "FR",
        defaultCarOccupancy: 2,
        includeRadiativeForcing: false,
        countBuildingEnergy: false,
        workingDaysPerYear: 220,
        factorYear: 2025,
      },
    };
    const repo = new TauriProfileSettingsRepository();
    await repo.set("p1", settings);

    expect(received).toEqual({ profileId: "p1", settings: backendDto });
  });
});
