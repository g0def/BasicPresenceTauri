import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type {
  BundleExportSummary,
  BundleImportSummary,
  BundleImportTarget,
  BundleSelection,
  ConflictStrategy,
  InspectedBundle,
} from "@/features/profile-transfer/domain/entities/bundle";
import type { TransferRepository } from "@/features/profile-transfer/domain/repositories/transfer-repository";

/**
 * TransferRepository backed by the bundle IPC commands. The Rust backend shows
 * the native file dialogs and reads/writes the files (where the encrypted data
 * lives): the WebView never sees or sends a file path — an inspected bundle is
 * referred to by an opaque, single-use handle.
 */
export class TauriTransferRepository implements TransferRepository {
  async exportProfileBundle(
    profileId: string,
    options: BundleSelection,
    defaultFileName: string,
  ): Promise<BundleExportSummary | null> {
    return invoke(COMMANDS.exportProfileBundle, {
      profileId,
      options,
      defaultFileName,
    });
  }

  async pickAndInspect(): Promise<InspectedBundle | null> {
    return invoke(COMMANDS.inspectProfileBundle);
  }

  async importProfileBundle(
    handle: string,
    selection: BundleSelection,
    target: BundleImportTarget,
    conflictStrategy: ConflictStrategy,
  ): Promise<BundleImportSummary> {
    return invoke(COMMANDS.importProfileBundle, {
      handle,
      selection,
      target,
      conflictStrategy,
    });
  }
}
