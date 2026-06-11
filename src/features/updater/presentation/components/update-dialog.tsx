import { useTranslation } from "react-i18next";
import { DownloadCloud } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useUpdater } from "../hooks/use-updater";

export function UpdateDialog() {
  const { t } = useTranslation();
  const { phase, update, progress, confirmInstall, relaunch, dismiss } =
    useUpdater();

  const open =
    phase === "available" ||
    phase === "downloading" ||
    phase === "installed" ||
    phase === "upToDate" ||
    phase === "error";

  const percent =
    progress && progress.total
      ? Math.round((progress.downloaded / progress.total) * 100)
      : null;

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (!o && phase !== "downloading") dismiss();
      }}
    >
      <DialogContent showCloseButton={phase !== "downloading"}>
        <DialogHeader>
          <DialogTitle>
            {phase === "available" &&
              t("updater.availableTitle", {
                version: update?.version ?? "",
              })}
            {phase === "downloading" &&
              t("updater.downloading", { percent: percent ?? "…" })}
            {phase === "installed" && t("updater.installed")}
            {phase === "upToDate" && t("updater.upToDate")}
            {phase === "error" && t("updater.error")}
          </DialogTitle>
          {phase === "available" && update?.body && (
            <DialogDescription className="whitespace-pre-wrap">
              {update.body}
            </DialogDescription>
          )}
          {phase === "available" && (
            <DialogDescription>
              {t("updater.availableBody", {
                version: update?.version ?? "",
                current: update?.currentVersion ?? "",
              })}
            </DialogDescription>
          )}
        </DialogHeader>

        {phase === "downloading" && (
          <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
            <div
              className="h-full bg-primary transition-all"
              style={{ width: percent !== null ? `${percent}%` : "40%" }}
            />
          </div>
        )}

        <DialogFooter>
          {phase === "available" && (
            <>
              <Button variant="outline" onClick={dismiss}>
                {t("updater.later")}
              </Button>
              <Button onClick={() => void confirmInstall()}>
                <DownloadCloud className="mr-2 size-4" />
                {t("updater.download")}
              </Button>
            </>
          )}
          {phase === "installed" && (
            <Button onClick={() => void relaunch()}>
              {t("updater.restart")}
            </Button>
          )}
          {(phase === "upToDate" || phase === "error") && (
            <Button variant="outline" onClick={dismiss}>
              {t("updater.later")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
