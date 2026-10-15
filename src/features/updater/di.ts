// Ports this feature takes from the DI container (see core/di).
import type { UpdaterRepository } from "@/features/updater/domain/repositories/updater-repository";

declare module "@/core/di" {
  interface Container {
    updaterRepository: UpdaterRepository;
  }
}
