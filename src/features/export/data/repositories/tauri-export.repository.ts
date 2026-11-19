import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type {
  AttendanceDay,
  AttendanceExportRequest,
  AttendanceExportSummary,
} from "@/features/export/domain/entities/attendance-export";
import type {
  ExportLabels,
  ExportOptions,
  ExportSummary,
} from "@/features/export/domain/entities/export";
import type { ExportRepository } from "@/features/export/domain/repositories/export-repository";

/**
 * ExportRepository backed by the `export_profile_data` IPC command. The Rust
 * backend shows the native "save as" dialog itself and writes the file: the
 * WebView only suggests a file name and never handles a file path (a
 * compromised WebView cannot write anywhere it likes).
 */
export class TauriExportRepository implements ExportRepository {
  async exportProfileData(
    profileId: string,
    options: ExportOptions,
    labels: ExportLabels,
    defaultFileName: string,
  ): Promise<ExportSummary | null> {
    // `null` when the user dismissed the dialog.
    return invoke(COMMANDS.exportProfileData, {
      profileId,
      options,
      labels,
      defaultFileName,
    });
  }

  async listAttendanceDays(profileId: string): Promise<AttendanceDay[]> {
    return invoke(COMMANDS.listAttendanceDays, { profileId });
  }

  async exportAttendance(
    profileId: string,
    request: AttendanceExportRequest,
    defaultFileName: string,
  ): Promise<AttendanceExportSummary | null> {
    // `null` when the user dismissed the dialog.
    return invoke(COMMANDS.exportAttendance, {
      profileId,
      request,
      defaultFileName,
    });
  }
}
