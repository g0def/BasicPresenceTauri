// Ports this feature takes from the DI container (see core/di).
import type { Co2ReferentialRepository } from "@/features/methodology/domain/repositories/co2-referential-repository";

declare module "@/core/di" {
  interface Container {
    co2ReferentialRepository: Co2ReferentialRepository;
  }
}
