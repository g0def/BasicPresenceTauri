import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { AvailableUpdate } from "@/features/updater/domain/entities/update";
import { UpdaterContext } from "@/features/updater/presentation/context/updater-context";
import type { UpdaterContextValue } from "@/features/updater/presentation/context/updater-context";
import { UpdateDialog } from "@/features/updater/presentation/components/update-dialog";

const SAMPLE_UPDATE: AvailableUpdate = {
  version: "9.9.9",
  currentVersion: "1.0.0",
  body: null,
  date: null,
};

/** Render the dialog on an available update, under a fake updater context. */
function renderAvailable(packageManaged: boolean) {
  const value: UpdaterContextValue = {
    phase: "available",
    update: SAMPLE_UPDATE,
    progress: null,
    packageManaged,
    checkForUpdates: vi.fn().mockResolvedValue(undefined),
    openUpdater: vi.fn(),
    confirmInstall: vi.fn().mockResolvedValue(undefined),
    relaunch: vi.fn().mockResolvedValue(undefined),
    dismiss: vi.fn(),
  };
  render(
    <UpdaterContext.Provider value={value}>
      <UpdateDialog />
    </UpdaterContext.Provider>,
  );
}

describe("UpdateDialog", () => {
  it("offers to download and install a regular installation", () => {
    renderAvailable(false);

    expect(
      screen.getByRole("button", { name: /Télécharger et installer/ }),
    ).toBeInTheDocument();
    expect(screen.queryByText("sudo pacman -Syu")).not.toBeInTheDocument();
  });

  it("points a pacman-managed installation to pacman instead of installing", () => {
    renderAvailable(true);

    expect(screen.getByText("sudo pacman -Syu")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Télécharger et installer/ }),
    ).not.toBeInTheDocument();
  });
});
