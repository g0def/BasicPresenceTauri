import { mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it, vi } from "vitest";

import type {
  BundleImportTarget,
  BundleManifest,
  BundleSelection,
} from "@/features/profile-transfer/domain/entities/bundle";
import { TauriTransferRepository } from "@/features/profile-transfer/data/repositories/tauri-transfer.repository";

// The repository is the only place that drives the native dialogs; stub them so
// the test controls "picked path" vs "cancelled" without a real OS dialog.
vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(),
  open: vi.fn(),
}));
import { open, save } from "@tauri-apps/plugin-dialog";

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

afterEach(() => {
  vi.clearAllMocks();
});

describe("TauriTransferRepository", () => {
  it("export: opens the save dialog then invokes export_profile_bundle with camelCase args", async () => {
    vi.mocked(save).mockResolvedValue("/tmp/profile.json");
    const summary = {
      days: 2,
      trips: 1,
      workEntries: 0,
      notes: 0,
      taskPresets: 0,
      commutes: 0,
      settings: true,
      path: "/tmp/profile.json",
    };
    mockIPC((cmd, args) => {
      if (cmd === "export_profile_bundle") {
        expect(args).toEqual({
          profileId: "p1",
          options: SELECTION,
          path: "/tmp/profile.json",
        });
        return summary;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriTransferRepository();
    const result = await repo.exportProfileBundle(
      "p1",
      SELECTION,
      "profile.json",
    );

    expect(save).toHaveBeenCalledOnce();
    expect(result).toEqual(summary);
  });

  it("export: returns null (and never invokes) when the save dialog is cancelled", async () => {
    vi.mocked(save).mockResolvedValue(null);
    mockIPC(() => {
      throw new Error("invoke must not be called when the dialog is cancelled");
    });

    const repo = new TauriTransferRepository();
    expect(
      await repo.exportProfileBundle("p1", SELECTION, "profile.json"),
    ).toBeNull();
  });

  it("inspect: opens the file dialog then invokes inspect_profile_bundle and returns path + manifest", async () => {
    vi.mocked(open).mockResolvedValue("/tmp/in.json");
    mockIPC((cmd, args) => {
      if (cmd === "inspect_profile_bundle") {
        expect(args).toEqual({ path: "/tmp/in.json" });
        return MANIFEST;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const repo = new TauriTransferRepository();
    expect(await repo.pickAndInspect()).toEqual({
      path: "/tmp/in.json",
      manifest: MANIFEST,
    });
  });

  it("inspect: returns null when the open dialog is cancelled", async () => {
    vi.mocked(open).mockResolvedValue(null);
    const repo = new TauriTransferRepository();
    expect(await repo.pickAndInspect()).toBeNull();
  });

  it("import: invokes import_profile_bundle with path/selection/target/conflictStrategy", async () => {
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
          path: "/tmp/in.json",
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
      await repo.importProfileBundle("/tmp/in.json", SELECTION, target, "skip"),
    ).toEqual(summary);
  });
});
