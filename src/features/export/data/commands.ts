// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.
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

declare module "@/core/ipc" {
  interface CommandRegistry {
    export_profile_data: {
      args: {
        profileId: string;
        options: ExportOptions;
        labels: ExportLabels;
        defaultFileName: string;
      };
      result: ExportSummary | null;
    };
    list_attendance_days: {
      args: { profileId: string };
      result: AttendanceDay[];
    };
    export_attendance: {
      args: {
        profileId: string;
        request: AttendanceExportRequest;
        defaultFileName: string;
      };
      result: AttendanceExportSummary | null;
    };
  }
}
