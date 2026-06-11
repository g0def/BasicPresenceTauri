import type { AvailableUpdate, DownloadProgress } from "../entities/update";

export interface UpdaterRepository {
  check(): Promise<AvailableUpdate | null>;
  downloadAndInstall(
    onProgress: (progress: DownloadProgress) => void,
  ): Promise<void>;
  relaunch(): Promise<void>;
}
