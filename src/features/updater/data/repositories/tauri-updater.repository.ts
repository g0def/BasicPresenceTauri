import { check as tauriCheck, type Update } from "@tauri-apps/plugin-updater";
import { relaunch as tauriRelaunch } from "@tauri-apps/plugin-process";

import type {
  AvailableUpdate,
  DownloadProgress,
} from "@/features/updater/domain/entities/update";
import type { UpdaterRepository } from "@/features/updater/domain/repositories/updater-repository";

/**
 * Only file in the updater feature allowed to import @tauri-apps/plugin-updater
 * and @tauri-apps/plugin-process directly.
 */
export class TauriUpdaterRepository implements UpdaterRepository {
  // Hold the Update handle between check() and downloadAndInstall().
  private pendingUpdate: Update | null = null;

  async check(): Promise<AvailableUpdate | null> {
    const update = await tauriCheck();
    if (!update) {
      this.pendingUpdate = null;
      return null;
    }
    this.pendingUpdate = update;
    return {
      version: update.version,
      currentVersion: update.currentVersion,
      body: update.body ?? null,
      date: update.date ?? null,
    };
  }

  async downloadAndInstall(
    onProgress: (progress: DownloadProgress) => void,
  ): Promise<void> {
    if (!this.pendingUpdate) {
      throw new Error("No pending update — call check() first.");
    }
    let downloaded = 0;
    // The size is only announced once, on "Started": keep it for every
    // "Progress" event so the progress bar can show a percentage.
    let total: number | null = null;
    await this.pendingUpdate.downloadAndInstall((event) => {
      if (event.event === "Started") {
        downloaded = 0;
        total = event.data.contentLength ?? null;
        onProgress({ downloaded: 0, total });
      } else if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        onProgress({ downloaded, total });
      }
    });
    this.pendingUpdate = null;
  }

  async relaunch(): Promise<void> {
    await tauriRelaunch();
  }
}
