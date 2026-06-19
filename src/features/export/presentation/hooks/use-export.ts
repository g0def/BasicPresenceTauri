import { useCallback, useState } from "react";

import { isAppError } from "@/core/errors";
import { TauriExportRepository } from "@/features/export/data/repositories/tauri-export.repository";
import type {
  ExportLabels,
  ExportOptions,
  ExportSummary,
} from "@/features/export/domain/entities/export";

// Single repository instance — the export action is stateless on its own.
const repo = new TauriExportRepository();

/**
 * Drives a one-shot data export: opens the save dialog + writes the file,
 * exposing `isExporting`/`error` for the dialog UI. Returns the run summary, or
 * `null` when the user cancelled the dialog (no error in that case).
 */
export function useExport() {
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const exportData = useCallback(
    async (
      profileId: string,
      options: ExportOptions,
      labels: ExportLabels,
      defaultFileName: string,
    ): Promise<ExportSummary | null> => {
      setIsExporting(true);
      setError(null);
      try {
        return await repo.exportProfileData(
          profileId,
          options,
          labels,
          defaultFileName,
        );
      } catch (e) {
        setError(isAppError(e) ? e.message : "Une erreur est survenue");
        return null;
      } finally {
        setIsExporting(false);
      }
    },
    [],
  );

  const clearError = useCallback(() => setError(null), []);

  return { exportData, isExporting, error, clearError };
}
