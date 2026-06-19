import type {
  BundleExportSummary,
  BundleImportSummary,
  BundleImportTarget,
  BundleManifest,
  BundleSelection,
  ConflictStrategy,
} from "@/features/profile-transfer/domain/entities/bundle";

export interface TransferRepository {
  /**
   * Open a native "save as" dialog, then write the profile as a shareable JSON
   * bundle at the chosen path. Resolves to `null` when the user cancels the
   * dialog (throws an {@link AppError} on a backend failure).
   */
  exportProfileBundle(
    profileId: string,
    options: BundleSelection,
    defaultFileName: string,
  ): Promise<BundleExportSummary | null>;

  /**
   * Open a native "open file" dialog, then inspect the chosen bundle. Resolves
   * to the picked path plus its manifest, or `null` when the user cancels.
   */
  pickAndInspect(): Promise<{ path: string; manifest: BundleManifest } | null>;

  /** Import the selected categories from a previously-inspected bundle. */
  importProfileBundle(
    path: string,
    selection: BundleSelection,
    target: BundleImportTarget,
    conflictStrategy: ConflictStrategy,
  ): Promise<BundleImportSummary>;
}
