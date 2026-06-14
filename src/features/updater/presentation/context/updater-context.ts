import { createContext } from "react";

import type {
  AvailableUpdate,
  DownloadProgress,
  UpdatePhase,
} from "../../domain/entities/update";

export interface UpdaterContextValue {
  phase: UpdatePhase;
  update: AvailableUpdate | null;
  progress: DownloadProgress | null;
  checkForUpdates: (manual?: boolean) => Promise<void>;
  /** Open the updater from the header badge: re-open the dialog if an update is
   * already known, otherwise trigger a manual check. */
  openUpdater: () => void;
  confirmInstall: () => Promise<void>;
  relaunch: () => Promise<void>;
  dismiss: () => void;
}

export const UpdaterContext = createContext<UpdaterContextValue | null>(null);
