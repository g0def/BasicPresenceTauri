import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";

import { useContainer } from "@/core/di";
import type {
  AvailableUpdate,
  DownloadProgress,
  UpdatePhase,
} from "@/features/updater/domain/entities/update";
import { UpdaterContext } from "@/features/updater/presentation/context/updater-context";

interface UpdaterProviderProps {
  children: ReactNode;
  /** Run right before the app restarts into the installed update (e.g. log
   * out, so the vault is locked and its integrity baseline recorded). Injected
   * by the composition root: the updater does not know about auth. */
  beforeRelaunch?: () => Promise<void>;
}

export function UpdaterProvider({
  children,
  beforeRelaunch,
}: UpdaterProviderProps) {
  const { updaterRepository: repo } = useContainer();
  const [phase, setPhase] = useState<UpdatePhase>("idle");
  const [update, setUpdate] = useState<AvailableUpdate | null>(null);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);

  // Prevent concurrent checks.
  const checking = useRef(false);

  const checkForUpdates = useCallback(
    async (manual = false) => {
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
    },
    [repo],
  );

  const confirmInstall = useCallback(async () => {
    setPhase("downloading");
    setProgress({ downloaded: 0, total: null });
    try {
      await repo.downloadAndInstall((p) => setProgress(p));
      setPhase("installed");
    } catch {
      setPhase("error");
    }
  }, [repo]);

  const relaunch = useCallback(async () => {
    await beforeRelaunch?.();
    await repo.relaunch();
  }, [beforeRelaunch, repo]);

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
  // installs; the user triggers the install manually from the badge. An
  // effect (not render-time side effect); the ref keeps StrictMode's double
  // mount from checking twice.
  const startupDone = useRef(false);
  useEffect(() => {
    if (startupDone.current || !import.meta.env.PROD) return;
    startupDone.current = true;
    void checkForUpdates(false);
  }, [checkForUpdates]);

  const value = useMemo(
    () => ({
      phase,
      update,
      progress,
      checkForUpdates,
      openUpdater,
      confirmInstall,
      relaunch,
      dismiss,
    }),
    [
      phase,
      update,
      progress,
      checkForUpdates,
      openUpdater,
      confirmInstall,
      relaunch,
      dismiss,
    ],
  );

  return (
    <UpdaterContext.Provider value={value}>{children}</UpdaterContext.Provider>
  );
}
