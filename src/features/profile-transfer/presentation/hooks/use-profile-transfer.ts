import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";

import { useContainer } from "@/core/di";
import { errorMessage, isSessionExpired, useCommand } from "@/core/query";
import type {
  BundleExportSummary,
  BundleImportSummary,
  BundleImportTarget,
  InspectedBundle,
  BundleSelection,
  ConflictStrategy,
} from "@/features/profile-transfer/domain/entities/bundle";

/**
 * Drives profile export/inspect/import: opens the native dialogs and invokes the
 * backend, exposing per-action loading flags + a shared `error` for the dialogs.
 * Each action returns `null` on cancel/failure (an error is set on failure only).
 */
export function useProfileTransfer() {
  const { t } = useTranslation();
  const { transferRepository: repo } = useContainer();
  // IPC goes through the command runner: SESSION_EXPIRED is handled globally.
  const { run } = useCommand();
  const [isExporting, setIsExporting] = useState(false);
  const [isInspecting, setIsInspecting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // A locked vault is not an error to show here: the app logs out.
  const report = useCallback(
    (e: unknown) => {
      if (!isSessionExpired(e)) setError(errorMessage(e, t("errors.generic")));
    },
    [t],
  );

  const exportBundle = useCallback(
    async (
      profileId: string,
      options: BundleSelection,
      defaultFileName: string,
    ): Promise<BundleExportSummary | null> => {
      setIsExporting(true);
      setError(null);
      try {
        return await run(() =>
          repo.exportProfileBundle(profileId, options, defaultFileName),
        );
      } catch (e) {
        report(e);
        return null;
      } finally {
        setIsExporting(false);
      }
    },
    [repo, run, report],
  );

  const inspectBundle =
    useCallback(async (): Promise<InspectedBundle | null> => {
      setIsInspecting(true);
      setError(null);
      try {
        return await run(() => repo.pickAndInspect());
      } catch (e) {
        report(e);
        return null;
      } finally {
        setIsInspecting(false);
      }
    }, [repo, run, report]);

  const importBundle = useCallback(
    async (
      handle: string,
      selection: BundleSelection,
      target: BundleImportTarget,
      conflictStrategy: ConflictStrategy,
    ): Promise<BundleImportSummary | null> => {
      setIsImporting(true);
      setError(null);
      try {
        return await run(() =>
          repo.importProfileBundle(handle, selection, target, conflictStrategy),
        );
      } catch (e) {
        report(e);
        return null;
      } finally {
        setIsImporting(false);
      }
    },
    [repo, run, report],
  );

  const clearError = useCallback(() => setError(null), []);

  return {
    exportBundle,
    inspectBundle,
    importBundle,
    isExporting,
    isInspecting,
    isImporting,
    error,
    clearError,
  };
}
