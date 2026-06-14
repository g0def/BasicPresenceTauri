import { useTranslation } from "react-i18next";
import { DownloadCloud, Loader2 } from "lucide-react";

import { cn } from "@/lib/utils";
import { useUpdater } from "../hooks/use-updater";

/**
 * Version pill next to the "Basic Presence" wordmark. Shows the running version
 * (injected at build time from package.json), and acts as the entry point to the
 * updater: clicking it re-opens the update dialog when an update is known, or
 * runs a manual check otherwise. When an update is available it switches to the
 * warning (ochre) palette so it stands out without the urgency of destructive.
 */
export function VersionBadge() {
  const { t } = useTranslation();
  const { phase, update, openUpdater } = useUpdater();

  const outdated = update !== null && phase !== "installed";
  const busy = phase === "checking" || phase === "downloading";

  return (
    <button
      type="button"
      onClick={openUpdater}
      disabled={busy}
      title={
        outdated
          ? t("updater.badgeOutdated", { version: update?.version ?? "" })
          : t("updater.badgeUpToDate", { version: __APP_VERSION__ })
      }
      aria-label={
        outdated
          ? t("updater.badgeOutdated", { version: update?.version ?? "" })
          : t("updater.badgeUpToDate", { version: __APP_VERSION__ })
      }
      className={cn(
        "flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-semibold transition-colors disabled:opacity-60",
        outdated
          ? "border-warning/30 bg-warning/10 text-warning hover:bg-warning/20"
          : "border-border bg-muted/40 text-muted-foreground hover:bg-muted",
      )}
    >
      {busy ? (
        <Loader2 className="size-3 animate-spin" />
      ) : outdated ? (
        <DownloadCloud className="size-3" />
      ) : null}
      <span>v{__APP_VERSION__}</span>
    </button>
  );
}
