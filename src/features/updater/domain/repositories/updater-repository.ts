import type {
  AvailableUpdate,
  DownloadProgress,
} from "@/features/updater/domain/entities/update";

export interface UpdaterRepository {
  check(): Promise<AvailableUpdate | null>;
  downloadAndInstall(
    onProgress: (progress: DownloadProgress) => void,
  ): Promise<void>;
  relaunch(): Promise<void>;
  /** True when a system package manager (pacman) owns the installation: the
   * update must then go through it, never through `downloadAndInstall`. */
  isPackageManaged(): Promise<boolean>;
}
