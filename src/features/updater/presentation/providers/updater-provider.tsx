import { useCallback, useRef, useState, type ReactNode } from "react";

import { TauriUpdaterRepository } from "../../data/repositories/tauri-updater.repository";
import type {
  AvailableUpdate,
  DownloadProgress,
  UpdatePhase,
} from "../../domain/entities/update";
import type { UpdaterRepository } from "../../domain/repositories/updater-repository";
import { UpdaterContext } from "../context/updater-context";

const repo: UpdaterRepository = new TauriUpdaterRepository();

export function UpdaterProvider({ children }: { children: ReactNode }) {
  const [phase, setPhase] = useState<UpdatePhase>("idle");
  const [update, setUpdate] = useState<AvailableUpdate | null>(null);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);

  // Prevent concurrent checks.
  const checking = useRef(false);

  const checkForUpdates = useCallback(async (manual = false) => {
    if (checking.current) return;
    checking.current = true;
    // The check always runs automatically (startup) or manually (badge). A
    // silent startup check stays invisible: it only records the update so the
    // header badge can flag it — installing is always a manual action. Only a
    // manual check surfaces the "checking…"/dialog flow.
    if (manual) setPhase("checking");
    try {
      const found = await repo.check();
      if (found) {
        setUpdate(found);
        // Open the dialog only on a manual check; a silent check just keeps
        // `update` set so the badge turns warning, never auto-installs.
        if (manual) setPhase("available");
      } else {
        setUpdate(null);
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
    setProgress(null);
    // Keep `update` so the header badge keeps flagging the available update
    // after the user clicks "Later" — it clears only once the update installs.
  }, []);

  // Open the updater from the header badge: re-open the dialog when an update is
  // already known (no re-check), otherwise run a manual check.
  const openUpdater = useCallback(() => {
    if (update) setPhase("available");
    else void checkForUpdates(true);
  }, [update, checkForUpdates]);

  // Startup check: run once in production so the header badge can flag an
  // available update. It is always silent — never auto-opens the dialog nor
  // installs; the user triggers the install manually from the badge.
  // Using a ref so it only fires once per mount without useEffect dependencies.
  const startupDone = useRef(false);
  if (!startupDone.current && import.meta.env.PROD) {
    startupDone.current = true;
    void checkForUpdates(false);
  }

  return (
    <UpdaterContext.Provider
      value={{
        phase,
        update,
        progress,
        checkForUpdates,
        openUpdater,
        confirmInstall,
        relaunch,
        dismiss,
      }}
    >
      {children}
    </UpdaterContext.Provider>
  );
}
