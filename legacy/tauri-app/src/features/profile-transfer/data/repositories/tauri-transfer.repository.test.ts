import { mockIPC } from "@tauri-apps/api/mocks";
import { describe, expect, it } from "vitest";

import type {
  BundleImportTarget,
  BundleManifest,
  BundleSelection,
} from "@/features/profile-transfer/domain/entities/bundle";
import { TauriTransferRepository } from "@/features/profile-transfer/data/repositories/tauri-transfer.repository";

// The native dialogs are opened by the Rust backend: the repository only sends
// a suggested file name / an opaque handle — never a file path.

const SELECTION: BundleSelection = {
  includeDays: true,
  includeTrips: true,
  includeWorkHours: true,
  includeNotes: true,
  includeTaskPresets: true,
  includeCommutes: true,
  includeSettings: true,
};

const MANIFEST: BundleManifest = {
  version: 1,
  compatible: true,
  exportedAt: 1_717_200_000_000,
  app: "0.2.0",
  profileFirstName: "Ada",
  profileLastName: "Lovelace",
  profileEnterprise: "Analytical Engine",
  days: 2,
  trips: 1,
  workEntries: 0,
  notes: 0,
  taskPresets: 0,
  commutes: 0,
  hasSettings: true,
  unknownModeIds: [],
};

describe("TauriTransferRepository", () => {
  it("export: sends a suggested file name (no path) to export_profile_bundle", async () => {
    const summary = {
      days: 2,
      trips: 1,
      workEntries: 0,
      notes: 0,
      taskPresets: 0,
      commutes: 0,
      settings: true,
      path: "/home/u/profile.json",
    };
    mockIPC((cmd, args) => {
      if (cmd === "export_profile_bundle") {
        expect(args).toEqual({
          profileId: "p1",
          options: SELECTION,
          defaultFileName: "profile.json",
        });
        return summary;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriTransferRepository();
    expect(
      await repo.exportProfileBundle("p1", SELECTION, "profile.json"),
    ).toEqual(summary);
  });

  it("export: returns null when the user cancels the backend dialog", async () => {
    mockIPC(() => null);
    const repo = new TauriTransferRepository();
    expect(
      await repo.exportProfileBundle("p1", SELECTION, "profile.json"),
    ).toBeNull();
  });

  it("inspect: returns the opaque handle + manifest picked by the backend", async () => {
    mockIPC((cmd, args) => {
      if (cmd === "inspect_profile_bundle") {
        expect(args).toEqual({});
        return { handle: "h1", manifest: MANIFEST };
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriTransferRepository();
    expect(await repo.pickAndInspect()).toEqual({
      handle: "h1",
      manifest: MANIFEST,
    });
  });

  it("inspect: returns null when the open dialog is cancelled", async () => {
    mockIPC(() => null);
    const repo = new TauriTransferRepository();
    expect(await repo.pickAndInspect()).toBeNull();
  });

  it("import: invokes import_profile_bundle with handle/selection/target/conflictStrategy", async () => {
    const target: BundleImportTarget = { kind: "new" };
    const summary = {
      profileId: "new1",
      daysImported: 2,
      daysReplaced: 0,
      daysSkipped: 0,
      trips: 1,
      workEntries: 0,
      notes: 0,
      taskPresets: 0,
      commutes: 0,
      settings: true,
    };
    mockIPC((cmd, args) => {
      if (cmd === "import_profile_bundle") {
        expect(args).toEqual({
          handle: "h1",
          selection: SELECTION,
          target,
          conflictStrategy: "skip",
        });
        return summary;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriTransferRepository();
    expect(
      await repo.importProfileBundle("h1", SELECTION, target, "skip"),
    ).toEqual(summary);
  });
});
