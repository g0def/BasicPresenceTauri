import { open, save } from "@tauri-apps/plugin-dialog";

import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type {
  BundleExportSummary,
  BundleImportSummary,
  BundleImportTarget,
  BundleManifest,
  BundleSelection,
  ConflictStrategy,
} from "@/features/profile-transfer/domain/entities/bundle";
import type { TransferRepository } from "@/features/profile-transfer/domain/repositories/transfer-repository";

const JSON_FILTER = [{ name: "Profil", extensions: ["json"] }];

/**
 * TransferRepository backed by the Tauri dialog plugin (native path picking)
 * plus the bundle IPC commands (the file read/write happens in Rust, where the
 * encrypted data lives). Only file in this feature allowed to import
 * `@tauri-apps/plugin-dialog` directly.
 */
export class TauriTransferRepository implements TransferRepository {
  async exportProfileBundle(
    profileId: string,
    options: BundleSelection,
    defaultFileName: string,
  ): Promise<BundleExportSummary | null> {
    const path = await save({
      defaultPath: defaultFileName,
      filters: JSON_FILTER,
    });
    if (!path) return null;
    return invoke<BundleExportSummary>(COMMANDS.exportProfileBundle, {
      profileId,
      options,
      path,
    });
  }

  async pickAndInspect(): Promise<{
    path: string;
    manifest: BundleManifest;
  } | null> {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: JSON_FILTER,
    });
    const path = Array.isArray(selected) ? selected[0] : selected;
    if (!path) return null;
    const manifest = await invoke<BundleManifest>(
      COMMANDS.inspectProfileBundle,
      { path },
    );
    return { path, manifest };
  }

  async importProfileBundle(
    path: string,
    selection: BundleSelection,
    target: BundleImportTarget,
    conflictStrategy: ConflictStrategy,
  ): Promise<BundleImportSummary> {
    return invoke<BundleImportSummary>(COMMANDS.importProfileBundle, {
      path,
      selection,
      target,
      conflictStrategy,
    });
  }
}
