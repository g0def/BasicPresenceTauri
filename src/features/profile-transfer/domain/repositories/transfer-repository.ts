import type {
  BundleExportSummary,
  BundleImportSummary,
  BundleImportTarget,
  InspectedBundle,
  BundleSelection,
  ConflictStrategy,
} from "@/features/profile-transfer/domain/entities/bundle";

export interface TransferRepository {
  /**
   * Have the backend open a native "save as" dialog, then write the profile as a shareable JSON
   * bundle at the chosen path. Resolves to `null` when the user cancels the
   * dialog (throws an {@link AppError} on a backend failure).
   */
  exportProfileBundle(
    profileId: string,
    options: BundleSelection,
    defaultFileName: string,
  ): Promise<BundleExportSummary | null>;

  /**
   * Have the backend open a native "open file" dialog, then inspect the chosen
   * bundle. Resolves to the bundle's handle plus its manifest, or `null` when
   * the user cancels.
   */
  pickAndInspect(): Promise<InspectedBundle | null>;

  /** Import the selected categories from a previously-inspected bundle. */
  importProfileBundle(
    handle: string,
    selection: BundleSelection,
    target: BundleImportTarget,
    conflictStrategy: ConflictStrategy,
  ): Promise<BundleImportSummary>;
}
