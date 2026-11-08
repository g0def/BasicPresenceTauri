import type {
  ExportLabels,
  ExportOptions,
  ExportSummary,
} from "@/features/export/domain/entities/export";

export interface ExportRepository {
  /**
   * Open a native "save as" dialog, then write the profile's data as a
   * multi-sheet .ods file at the chosen path. `labels` carries the translated
   * sheet/column text. Resolves to `null` when the user cancels the dialog (and
   * throws an {@link AppError} on a backend failure).
   */
  exportProfileData(
    profileId: string,
    options: ExportOptions,
    labels: ExportLabels,
    defaultFileName: string,
  ): Promise<ExportSummary | null>;
}
