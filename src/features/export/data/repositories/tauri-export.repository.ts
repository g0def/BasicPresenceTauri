import { save } from "@tauri-apps/plugin-dialog";

import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type {
  ExportLabels,
  ExportOptions,
  ExportSummary,
} from "@/features/export/domain/entities/export";
import type { ExportRepository } from "@/features/export/domain/repositories/export-repository";

/**
 * ExportRepository backed by the Tauri dialog plugin (native path picking) plus
 * the `export_profile_data` IPC command (the actual file writing happens in the
 * Rust backend, where the encrypted data lives).
 *
 * Only file in the export feature allowed to import `@tauri-apps/plugin-dialog`
 * directly (same convention as the updater repository).
 */
export class TauriExportRepository implements ExportRepository {
  async exportProfileData(
    profileId: string,
    options: ExportOptions,
    labels: ExportLabels,
    defaultFileName: string,
  ): Promise<ExportSummary | null> {
    const path = await save({
      defaultPath: defaultFileName,
      filters: [{ name: "OpenDocument", extensions: ["ods"] }],
    });
    // `save` returns null when the user dismisses the dialog.
    if (!path) return null;
    return invoke<ExportSummary>(COMMANDS.exportProfileData, {
      profileId,
      options,
      labels,
      path,
    });
  }
}
