import { mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { DownloadProgress } from "@/features/updater/domain/entities/update";

type DownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished" };

const { check, relaunch } = vi.hoisted(() => ({
  check: vi.fn(),
  relaunch: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-updater", () => ({ check }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch }));

import { TauriUpdaterRepository } from "@/features/updater/data/repositories/tauri-updater.repository";

afterEach(() => vi.clearAllMocks());

describe("TauriUpdaterRepository", () => {
  it("reports progress against the size announced once at start", async () => {
    check.mockResolvedValue({
      version: "1.0.1",
      currentVersion: "1.0.0",
      body: null,
      date: null,
      downloadAndInstall: (cb: (e: DownloadEvent) => void) => {
        cb({ event: "Started", data: { contentLength: 100 } });
        cb({ event: "Progress", data: { chunkLength: 40 } });
        cb({ event: "Progress", data: { chunkLength: 60 } });
        cb({ event: "Finished" });
        return Promise.resolve();
      },
    });

    const repo = new TauriUpdaterRepository();
    expect(await repo.check()).toMatchObject({ version: "1.0.1" });
    const seen: DownloadProgress[] = [];
    await repo.downloadAndInstall((p) => seen.push(p));

    expect(seen).toEqual([
      { downloaded: 0, total: 100 },
      { downloaded: 40, total: 100 },
      { downloaded: 100, total: 100 },
    ]);
  });

  it("asks the backend whether pacman owns the installation", async () => {
    mockIPC((cmd) => {
      if (cmd === "is_package_managed") return true;
      throw new Error(`unexpected command: ${cmd}`);
    });

    expect(await new TauriUpdaterRepository().isPackageManaged()).toBe(true);
  });

  it("refuses to install without a prior successful check", async () => {
    check.mockResolvedValue(null);
    const repo = new TauriUpdaterRepository();
    expect(await repo.check()).toBeNull();
    await expect(repo.downloadAndInstall(() => {})).rejects.toThrow();
  });
});
