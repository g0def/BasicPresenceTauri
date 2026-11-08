import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import { useContainer } from "@/core/di";
import { errorMessage, isSessionExpired, useCommand } from "@/core/query";
import type {
  ExportLabels,
  ExportOptions,
  ExportSummary,
} from "@/features/export/domain/entities/export";

/**
 * Drives a one-shot data export: the backend shows the save dialog and writes
 * the file; exposes `isExporting`/`error` for the dialog UI. Returns the run
 * summary, or `null` when the user cancelled the dialog (no error in that case).
 */
export function useExport() {
  const { t } = useTranslation();
  const { exportRepository } = useContainer();
  const { run, isPending: isExporting } = useCommand();
  const [error, setError] = useState<string | null>(null);

  const exportData = useCallback(
    async (
      profileId: string,
      options: ExportOptions,
      labels: ExportLabels,
      defaultFileName: string,
    ): Promise<ExportSummary | null> => {
      setError(null);
      try {
        return await run(() =>
          exportRepository.exportProfileData(
            profileId,
            options,
            labels,
            defaultFileName,
          ),
        );
      } catch (e) {
        if (!isSessionExpired(e))
          setError(errorMessage(e, t("errors.generic")));
        return null;
      }
    },
    [exportRepository, run, t],
  );

  const clearError = useCallback(() => setError(null), []);

  return { exportData, isExporting, error, clearError };
}
