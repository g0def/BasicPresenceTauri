import { useCallback, useRef, useState, type ReactNode } from "react";

import { TauriUpdaterRepository } from "../../data/repositories/tauri-updater.repository";
import type {
  AvailableUpdate,
  DownloadProgress,
  UpdatePhase,
} from "../../domain/entities/update";
import type { UpdaterRepository } from "../../domain/repositories/updater-repository";
import { UpdaterContext } from "../context/updater-context";
import { useAutoUpdatePreference } from "../hooks/use-auto-update-preference";

const repo: UpdaterRepository = new TauriUpdaterRepository();

export function UpdaterProvider({ children }: { children: ReactNode }) {
  const { enabled: autoUpdateEnabled, toggleEnabled: toggleAutoUpdate } =
    useAutoUpdatePreference();
  const [phase, setPhase] = useState<UpdatePhase>("idle");
  const [update, setUpdate] = useState<AvailableUpdate | null>(null);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);

  // Prevent concurrent checks.
  const checking = useRef(false);

  const checkForUpdates = useCallback(async (manual = false) => {
    if (checking.current) return;
    checking.current = true;
    setPhase("checking");
    try {
      const found = await repo.check();
      if (found) {
        setUpdate(found);
        setPhase("available");
      } else {
        setPhase(manual ? "upToDate" : "idle");
      }
    } catch {
      setPhase(manual ? "error" : "idle");
    } finally {
      checking.current = false;
    }
  }, []);

  const confirmInstall = useCallback(async () => {
    setPhase("downloading");
    setProgress({ downloaded: 0, total: null });
    try {
      await repo.downloadAndInstall((p) => setProgress(p));
      setPhase("installed");
    } catch {
      setPhase("error");
    }
  }, []);

  const relaunch = useCallback(() => repo.relaunch(), []);

  const dismiss = useCallback(() => {
    setPhase("idle");
    setUpdate(null);
    setProgress(null);
  }, []);

  // Startup check: run once in production when auto-update is enabled.
  // Using a ref so it only fires once per mount without useEffect dependencies.
  const startupDone = useRef(false);
  if (!startupDone.current && import.meta.env.PROD && autoUpdateEnabled) {
    startupDone.current = true;
    void checkForUpdates(false);
  }

  return (
    <UpdaterContext.Provider
      value={{
        phase,
        update,
        progress,
        autoUpdateEnabled,
        toggleAutoUpdate,
        checkForUpdates,
        confirmInstall,
        relaunch,
        dismiss,
      }}
    >
      {children}
    </UpdaterContext.Provider>
  );
}
