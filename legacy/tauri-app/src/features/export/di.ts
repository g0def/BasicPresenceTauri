// Ports this feature takes from the DI container (see core/di).
import type { ExportRepository } from "@/features/export/domain/repositories/export-repository";

declare module "@/core/di" {
  interface Container {
    exportRepository: ExportRepository;
  }
}
