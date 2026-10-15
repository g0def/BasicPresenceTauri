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
}
