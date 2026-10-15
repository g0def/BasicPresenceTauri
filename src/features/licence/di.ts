// Ports this feature takes from the DI container (see core/di).
import type { LicenceRepository } from "@/features/licence/domain/repositories/licence-repository";

declare module "@/core/di" {
  interface Container {
    licenceRepository: LicenceRepository;
  }
}
