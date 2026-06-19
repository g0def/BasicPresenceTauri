import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AppError } from "@/core/errors";
import type {
  BundleExportSummary,
  BundleImportSummary,
  BundleImportTarget,
  BundleManifest,
  BundleSelection,
} from "@/features/profile-transfer/domain/entities/bundle";
import { useProfileTransfer } from "@/features/profile-transfer/presentation/hooks/use-profile-transfer";

// Mock the repository at the boundary the hook talks to. The hook instantiates a
// single TauriTransferRepository at module load, so the class returns an object
// backed by these shared spies that each test programs. `vi.hoisted` makes the
// spies available inside the hoisted `vi.mock` factory.
const { exportProfileBundle, pickAndInspect, importProfileBundle } = vi.hoisted(
  () => ({
    exportProfileBundle: vi.fn(),
    pickAndInspect: vi.fn(),
    importProfileBundle: vi.fn(),
  }),
);

vi.mock(
  "@/features/profile-transfer/data/repositories/tauri-transfer.repository",
  () => ({
    TauriTransferRepository: vi.fn(() => ({
      exportProfileBundle,
      pickAndInspect,
      importProfileBundle,
    })),
  }),
);

const SELECTION: BundleSelection = {
  includeDays: true,
  includeTrips: true,
  includeWorkHours: true,
  includeNotes: true,
  includeTaskPresets: true,
  includeCommutes: true,
  includeSettings: true,
};

const EXPORT_SUMMARY: BundleExportSummary = {
  days: 2,
  trips: 1,
  workEntries: 0,
  notes: 0,
  taskPresets: 0,
  commutes: 0,
  settings: true,
  path: "/tmp/profile.json",
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

const IMPORT_SUMMARY: BundleImportSummary = {
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

const TARGET: BundleImportTarget = { kind: "new" };

afterEach(() => {
  vi.clearAllMocks();
});

describe("useProfileTransfer", () => {
  describe("exportBundle", () => {
    it("returns the summary and toggles isExporting on success", async () => {
      let resolve: (s: BundleExportSummary) => void = () => {};
      exportProfileBundle.mockReturnValue(
        new Promise<BundleExportSummary>((r) => {
          resolve = r;
        }),
      );

      const { result } = renderHook(() => useProfileTransfer());

      let pending: Promise<BundleExportSummary | null>;
      act(() => {
        pending = result.current.exportBundle("p1", SELECTION, "profile.json");
      });

      await waitFor(() => expect(result.current.isExporting).toBe(true));

      await act(async () => {
        resolve(EXPORT_SUMMARY);
        await expect(pending).resolves.toEqual(EXPORT_SUMMARY);
      });

      expect(result.current.isExporting).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it("resolves to null, sets error, and resets isExporting on rejection", async () => {
      exportProfileBundle.mockRejectedValue(
        new AppError("EXPORT_FAILED", "export boom"),
      );

      const { result } = renderHook(() => useProfileTransfer());

      let outcome: BundleExportSummary | null = EXPORT_SUMMARY;
      await act(async () => {
        outcome = await result.current.exportBundle(
          "p1",
          SELECTION,
          "profile.json",
        );
      });

      expect(outcome).toBeNull();
      expect(result.current.error).toBe("export boom");
      expect(result.current.isExporting).toBe(false);
    });
  });

  describe("inspectBundle", () => {
    it("resolves to null, sets error, and resets isInspecting on rejection", async () => {
      pickAndInspect.mockRejectedValue(
        new AppError("INTERNAL", "inspect boom"),
      );

      const { result } = renderHook(() => useProfileTransfer());

      let outcome: { path: string; manifest: BundleManifest } | null = {
        path: "x",
        manifest: MANIFEST,
      };
      await act(async () => {
        outcome = await result.current.inspectBundle();
      });

      expect(outcome).toBeNull();
      expect(result.current.error).toBe("inspect boom");
      expect(result.current.isInspecting).toBe(false);
    });
  });

  describe("importBundle", () => {
    it("returns the summary and toggles isImporting on success", async () => {
      let resolve: (s: BundleImportSummary) => void = () => {};
      importProfileBundle.mockReturnValue(
        new Promise<BundleImportSummary>((r) => {
          resolve = r;
        }),
      );

      const { result } = renderHook(() => useProfileTransfer());

      let pending: Promise<BundleImportSummary | null>;
      act(() => {
        pending = result.current.importBundle(
          "/tmp/in.json",
          SELECTION,
          TARGET,
          "skip",
        );
      });

      await waitFor(() => expect(result.current.isImporting).toBe(true));

      await act(async () => {
        resolve(IMPORT_SUMMARY);
        await expect(pending).resolves.toEqual(IMPORT_SUMMARY);
      });

      expect(result.current.isImporting).toBe(false);
      expect(result.current.error).toBeNull();
    });

    it("resolves to null (does not throw), sets error, and resets isImporting on rejection", async () => {
      importProfileBundle.mockRejectedValue(
        new AppError("INTERNAL", "import boom"),
      );

      const { result } = renderHook(() => useProfileTransfer());

      let outcome: BundleImportSummary | null = IMPORT_SUMMARY;
      await act(async () => {
        outcome = await result.current.importBundle(
          "/tmp/in.json",
          SELECTION,
          TARGET,
          "skip",
        );
      });

      expect(outcome).toBeNull();
      expect(result.current.error).toBe("import boom");
      expect(result.current.isImporting).toBe(false);
    });
  });
});
