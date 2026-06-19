import { render, waitFor } from "@testing-library/react";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import type { Profile } from "@/features/profile/domain/entities/profile";
import {
  ProfileContext,
  type ProfileContextValue,
} from "@/features/profile/presentation/context/profile-context";
import { ProfileSettingsProvider } from "@/features/profile-settings/presentation/providers/profile-settings-provider";

const FONT_BY_PROFILE: Record<string, string> = { p1: "serif", p2: "mono" };

function settingsDto(profileId: string) {
  return {
    defaultStartMinutes: 510,
    noteFont: FONT_BY_PROFILE[profileId] ?? "sans",
    cellDisplayMode: "co2",
    gridCountry: "BE",
    defaultCarOccupancy: 1,
    includeRadiativeForcing: true,
    countBuildingEnergy: false,
    workingDaysPerYear: 220,
    factorYear: 2025,
  };
}

const profile = (id: string): Profile => ({
  id,
  firstName: "A",
  lastName: "B",
  enterprise: "C",
  poste: null,
  createdAt: 1,
  updatedAt: 1,
});

function profileCtx(active: Profile | null): ProfileContextValue {
  return {
    profiles: active ? [active] : [],
    activeProfile: active,
    isLoading: false,
    isSubmitting: false,
    error: null,
    createProfile: () => Promise.resolve(true),
    updateProfile: () => Promise.resolve(true),
    deleteProfile: () => Promise.resolve(),
    setActiveProfile: () => Promise.resolve(),
    refresh: () => Promise.resolve(),
    clearError: () => {},
  };
}

function renderWith(active: Profile | null) {
  return render(
    <ProfileContext.Provider value={profileCtx(active)}>
      <ProfileSettingsProvider>
        <div />
      </ProfileSettingsProvider>
    </ProfileContext.Provider>,
  );
}

describe("ProfileSettingsProvider note-font reconcile", () => {
  beforeEach(() => {
    localStorage.clear();
    // Skip the one-time legacy migration so loads don't trigger extra writes.
    localStorage.setItem("profile-settings-legacy-migrated", "1");
    delete document.documentElement.dataset.noteFont;
    mockIPC((cmd, args) => {
      if (cmd === "get_profile_settings") {
        return settingsDto((args as { profileId: string }).profileId);
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
  });

  afterEach(() => clearMocks());

  it("applies the active profile's font and updates it on profile switch", async () => {
    const { rerender } = renderWith(profile("p1"));
    await waitFor(() =>
      expect(document.documentElement.dataset.noteFont).toBe("serif"),
    );

    rerender(
      <ProfileContext.Provider value={profileCtx(profile("p2"))}>
        <ProfileSettingsProvider>
          <div />
        </ProfileSettingsProvider>
      </ProfileContext.Provider>,
    );
    await waitFor(() =>
      expect(document.documentElement.dataset.noteFont).toBe("mono"),
    );
  });

  it("falls back to the default font when no profile is active", async () => {
    renderWith(null);
    await waitFor(() =>
      expect(document.documentElement.dataset.noteFont).toBe("sans"),
    );
  });
});
