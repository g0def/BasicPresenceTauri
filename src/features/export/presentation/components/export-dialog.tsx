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
  DEFAULT_EXPORT_OPTIONS,
  type ExportOptions,
  type ExportSummary,
} from "@/features/export/domain/entities/export";
import { buildExportLabels } from "@/features/export/presentation/build-export-labels";
import { useExport } from "@/features/export/presentation/hooks/use-export";

interface ExportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Profile whose activity is exported (Settings only renders this with one). */
  profileId: string;
}

type Stage = "idle" | "result";

/** The option flags, in display order. The i18n key for each is
 *  `export.fields.<key>`. */
const OPTION_KEYS: (keyof ExportOptions)[] = [
  "includeCarbon",
  "includeHours",
  "includeTasks",
  "includeTrips",
  "includeNotes",
  "includeTimestamps",
];

/**
 * Export the active profile's data to a multi-sheet .ods file. Flow: tick the
 * data to include → pick a save location (native dialog) → the Rust backend
 * writes the file → show a summary.
 */
export function ExportDialog({
  open,
  onOpenChange,
  profileId,
}: ExportDialogProps) {
  const { t, i18n } = useTranslation();
  const { exportData, isExporting, error, clearError } = useExport();

  const [stage, setStage] = useState<Stage>("idle");
  const [options, setOptions] = useState<ExportOptions>(DEFAULT_EXPORT_OPTIONS);
  const [summary, setSummary] = useState<ExportSummary | null>(null);

  // Reset everything whenever the dialog (re)opens.
  useEffect(() => {
    if (!open) return;
    setStage("idle");
    setOptions(DEFAULT_EXPORT_OPTIONS);
    setSummary(null);
    clearError();
  }, [open, clearError]);

  const toggle = (key: keyof ExportOptions) =>
    setOptions((prev) => ({ ...prev, [key]: !prev[key] }));

  const runExport = async () => {
    const fileName = `${t("export.fileBaseName")}-${format(new Date(), "yyyy-MM-dd")}.ods`;
    // Labels are translated to the current language so the file matches the UI.
    const labels = buildExportLabels(t, i18n.resolvedLanguage ?? "fr");
    const result = await exportData(profileId, options, labels, fileName);
    if (result) {
      setSummary(result);
      setStage("result");
    }
    // `null` + an error → stay on the form (error shown below);
    // `null` + no error → the user cancelled the save dialog, stay idle.
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("export.title")}</DialogTitle>
          {stage === "idle" && (
            <DialogDescription>{t("export.description")}</DialogDescription>
          )}
        </DialogHeader>

        {stage === "idle" && (
          <div className="grid gap-2 text-sm">
            {OPTION_KEYS.map((key) => (
              <label key={key} className="flex items-center gap-2">
                <input
                  type="checkbox"
                  className="size-4"
                  checked={options[key]}
                  onChange={() => toggle(key)}
                />
                {t(`export.fields.${key}`)}
              </label>
            ))}
          </div>
        )}

        {stage === "result" && summary && (
          <div className="grid gap-2 text-sm">
            <p>
              {t("export.result", {
                days: summary.days,
                tasks: summary.tasks,
                trips: summary.trips,
              })}
            </p>
            <p className="break-all text-muted-foreground">{summary.path}</p>
          </div>
        )}

        {error && <p className="text-sm text-destructive">{error}</p>}

        <DialogFooter>
          {stage === "idle" && (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                {t("export.cancel")}
              </Button>
              <Button disabled={isExporting} onClick={() => void runExport()}>
                {isExporting ? t("export.exporting") : t("export.exportButton")}
              </Button>
            </>
          )}
          {stage === "result" && (
            <Button onClick={() => onOpenChange(false)}>
              {t("export.done")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
