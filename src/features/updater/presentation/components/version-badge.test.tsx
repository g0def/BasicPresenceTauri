import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { AvailableUpdate } from "../../domain/entities/update";
import { UpdaterContext } from "../context/updater-context";
import type { UpdaterContextValue } from "../context/updater-context";
import { VersionBadge } from "./version-badge";

/** Render the badge under a fake updater context so we drive its state directly
 * (no Tauri IPC). Returns the spies for click assertions. */
function renderBadge(overrides: Partial<UpdaterContextValue> = {}) {
  const value: UpdaterContextValue = {
    phase: "idle",
    update: null,
    progress: null,
    checkForUpdates: vi.fn().mockResolvedValue(undefined),
    openUpdater: vi.fn(),
    confirmInstall: vi.fn().mockResolvedValue(undefined),
    relaunch: vi.fn().mockResolvedValue(undefined),
    dismiss: vi.fn(),
    ...overrides,
  };
  render(
    <UpdaterContext.Provider value={value}>
      <VersionBadge />
    </UpdaterContext.Provider>,
  );
  return value;
}

const SAMPLE_UPDATE: AvailableUpdate = {
  version: "9.9.9",
  currentVersion: __APP_VERSION__,
  body: null,
  date: null,
};

describe("VersionBadge", () => {
  it("shows the running version and stays neutral when up to date", () => {
    renderBadge({ update: null, phase: "idle" });

    const badge = screen.getByRole("button");
    expect(badge).toHaveTextContent(`v${__APP_VERSION__}`);
    expect(badge).toHaveAccessibleName(/à jour$/);
    expect(badge).toBeEnabled();
    // Neutral palette, not the warning (outdated) one.
    expect(badge.className).not.toContain("text-warning");
  });

  it("turns warning and announces an available update when outdated", () => {
    renderBadge({ update: SAMPLE_UPDATE, phase: "idle" });

    const badge = screen.getByRole("button");
    expect(badge).toHaveAccessibleName(/^Mise à jour disponible/);
    expect(badge.className).toContain("text-warning");
    // The pill still shows the currently running version.
    expect(badge).toHaveTextContent(`v${__APP_VERSION__}`);
  });

  it("opens the updater when clicked", async () => {
    const user = userEvent.setup();
    const value = renderBadge({ update: SAMPLE_UPDATE, phase: "idle" });

    await user.click(screen.getByRole("button"));

    expect(value.openUpdater).toHaveBeenCalledTimes(1);
  });

  it("is disabled while a check or download is in flight", () => {
    renderBadge({ phase: "checking" });
    expect(screen.getByRole("button")).toBeDisabled();
  });
});
