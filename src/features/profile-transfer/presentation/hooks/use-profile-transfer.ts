import { useCallback, useState } from "react";

import { isAppError } from "@/core/errors";
import { TauriTransferRepository } from "@/features/profile-transfer/data/repositories/tauri-transfer.repository";
import type {
  BundleExportSummary,
  BundleImportSummary,
  BundleImportTarget,
  BundleManifest,
  BundleSelection,
  ConflictStrategy,
} from "@/features/profile-transfer/domain/entities/bundle";

// Single repository instance — the transfer actions are stateless on their own.
const repo = new TauriTransferRepository();

/**
 * Drives profile export/inspect/import: opens the native dialogs and invokes the
 * backend, exposing per-action loading flags + a shared `error` for the dialogs.
 * Each action returns `null` on cancel/failure (an error is set on failure only).
 */
export function useProfileTransfer() {
  const [isExporting, setIsExporting] = useState(false);
  const [isInspecting, setIsInspecting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const toMessage = (e: unknown) =>
    isAppError(e) ? e.message : "Une erreur est survenue";

  const exportBundle = useCallback(
    async (
      profileId: string,
      options: BundleSelection,
      defaultFileName: string,
    ): Promise<BundleExportSummary | null> => {
      setIsExporting(true);
      setError(null);
      try {
        return await repo.exportProfileBundle(
          profileId,
          options,
          defaultFileName,
        );
      } catch (e) {
        setError(toMessage(e));
        return null;
      } finally {
        setIsExporting(false);
      }
    },
    [],
  );

  const inspectBundle = useCallback(async (): Promise<{
    path: string;
    manifest: BundleManifest;
  } | null> => {
    setIsInspecting(true);
    setError(null);
    try {
      return await repo.pickAndInspect();
    } catch (e) {
      setError(toMessage(e));
      return null;
    } finally {
      setIsInspecting(false);
    }
  }, []);

  const importBundle = useCallback(
    async (
      path: string,
      selection: BundleSelection,
      target: BundleImportTarget,
      conflictStrategy: ConflictStrategy,
    ): Promise<BundleImportSummary | null> => {
      setIsImporting(true);
      setError(null);
      try {
        return await repo.importProfileBundle(
          path,
          selection,
          target,
          conflictStrategy,
        );
      } catch (e) {
        setError(toMessage(e));
        return null;
      } finally {
        setIsImporting(false);
      }
    },
    [],
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
