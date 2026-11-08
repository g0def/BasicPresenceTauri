// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.
import type {
  BundleExportSummary,
  BundleImportSummary,
  BundleImportTarget,
  BundleSelection,
  ConflictStrategy,
  InspectedBundle,
} from "@/features/profile-transfer/domain/entities/bundle";

declare module "@/core/ipc" {
  interface CommandRegistry {
    export_profile_bundle: {
      args: {
        profileId: string;
        options: BundleSelection;
        defaultFileName: string;
      };
      result: BundleExportSummary | null;
    };
    inspect_profile_bundle: { args: void; result: InspectedBundle | null };
    import_profile_bundle: {
      args: {
        handle: string;
        selection: BundleSelection;
        target: BundleImportTarget;
        conflictStrategy: ConflictStrategy;
      };
      result: BundleImportSummary;
    };
  }
}
