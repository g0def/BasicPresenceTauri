import { createContext } from "react";

import type { AvailableUpdate, DownloadProgress, UpdatePhase } from "../../domain/entities/update";

export interface UpdaterContextValue {
  phase: UpdatePhase;
  update: AvailableUpdate | null;
  progress: DownloadProgress | null;
  autoUpdateEnabled: boolean;
  toggleAutoUpdate: () => void;
  checkForUpdates: (manual?: boolean) => Promise<void>;
  confirmInstall: () => Promise<void>;
  relaunch: () => Promise<void>;
  dismiss: () => void;
}

export const UpdaterContext = createContext<UpdaterContextValue | null>(null);
