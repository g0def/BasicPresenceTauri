// Ports this feature takes from the DI container (see core/di).
import type { CommuteRepository } from "@/features/commute/domain/repositories/commute-repository";
import type { EmissionFactorRepository } from "@/features/commute/domain/repositories/emission-factor-repository";

declare module "@/core/di" {
  interface Container {
    commuteRepository: CommuteRepository;
    emissionFactorRepository: EmissionFactorRepository;
  }
}
