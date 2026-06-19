import { useEffect, useState } from "react";
import { format } from "date-fns";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  CATEGORY_KEYS,
  DAY_CHILD_KEYS,
  DEFAULT_SELECTION,
  type BundleExportSummary,
  type BundleSelection,
} from "@/features/profile-transfer/domain/entities/bundle";
import { useProfileTransfer } from "@/features/profile-transfer/presentation/hooks/use-profile-transfer";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";

interface ExportProfileDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Profile whose data is exported (Settings only renders this with one). */
  profileId: string;
}

type Stage = "idle" | "result";

/** A child category is disabled (and ignored) when the presence days are off. */
function isDisabled(key: (typeof CATEGORY_KEYS)[number], sel: BundleSelection) {
  return DAY_CHILD_KEYS.includes(key) && !sel.includeDays;
}

/**
 * Export a whole profile to a shareable JSON bundle. Flow: tick the categories
 * to include → pick a save location (native dialog) → the Rust backend writes
 * the file → show a summary. Sibling of the ODS ExportDialog.
 */
export function ExportProfileDialog({
  open,
  onOpenChange,
  profileId,
}: ExportProfileDialogProps) {
  const { t } = useTranslation();
  const { profiles } = useProfile();
  const { exportBundle, isExporting, error, clearError } = useProfileTransfer();

  const [stage, setStage] = useState<Stage>("idle");
  const [selection, setSelection] =
    useState<BundleSelection>(DEFAULT_SELECTION);
  const [summary, setSummary] = useState<BundleExportSummary | null>(null);

  useEffect(() => {
    if (!open) return;
    setStage("idle");
    setSelection(DEFAULT_SELECTION);
    setSummary(null);
    clearError();
  }, [open, clearError]);

  const toggle = (key: keyof BundleSelection) =>
    setSelection((prev) => ({ ...prev, [key]: !prev[key] }));

  const runExport = async () => {
    const profile = profiles.find((p) => p.id === profileId);
    const base = profile
      ? `${profile.firstName}-${profile.lastName}`.replace(/\s+/g, "-")
      : t("transfer.export.fileBaseName");
    const fileName = `${base}-${format(new Date(), "yyyy-MM-dd")}.json`;
    const result = await exportBundle(profileId, selection, fileName);
    if (result) {
      setSummary(result);
      setStage("result");
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("transfer.export.title")}</DialogTitle>
          {stage === "idle" && (
            <DialogDescription>
              {t("transfer.export.description")}
            </DialogDescription>
          )}
        </DialogHeader>

        {stage === "idle" && (
          <div className="grid gap-2 text-sm">
            {CATEGORY_KEYS.map((key) => {
              const disabled = isDisabled(key, selection);
              const child = DAY_CHILD_KEYS.includes(key);
              return (
                <label
                  key={key}
                  className={`flex items-center gap-2 ${child ? "pl-6" : ""} ${
                    disabled ? "opacity-50" : ""
                  }`}
                >
                  <input
                    type="checkbox"
                    className="size-4"
                    disabled={disabled}
                    checked={!disabled && selection[key]}
                    onChange={() => toggle(key)}
                  />
                  {t(`transfer.fields.${key}`)}
                </label>
              );
            })}
          </div>
        )}

        {stage === "result" && summary && (
          <div className="grid gap-2 text-sm">
            <p>
              {t("transfer.export.result", {
                days: summary.days,
                trips: summary.trips,
                workEntries: summary.workEntries,
                notes: summary.notes,
                taskPresets: summary.taskPresets,
                commutes: summary.commutes,
              })}
            </p>
            {summary.settings && <p>{t("transfer.export.resultSettings")}</p>}
            <p className="break-all text-muted-foreground">{summary.path}</p>
          </div>
        )}

        {error && <p className="text-sm text-destructive">{error}</p>}

        <DialogFooter>
          {stage === "idle" && (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                {t("transfer.export.cancel")}
              </Button>
              <Button disabled={isExporting} onClick={() => void runExport()}>
                {isExporting
                  ? t("transfer.export.exporting")
                  : t("transfer.export.exportButton")}
              </Button>
            </>
          )}
          {stage === "result" && (
            <Button onClick={() => onOpenChange(false)}>
              {t("transfer.export.done")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
