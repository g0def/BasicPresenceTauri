// Ports this feature takes from the DI container (see core/di).
import type { PresenceRepository } from "@/features/presence/domain/repositories/presence-repository";

declare module "@/core/di" {
  interface Container {
    presenceRepository: PresenceRepository;
  }
}
