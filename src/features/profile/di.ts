// Ports this feature takes from the DI container (see core/di).
import type { ProfileRepository } from "@/features/profile/domain/repositories/profile-repository";

declare module "@/core/di" {
  interface Container {
    profileRepository: ProfileRepository;
  }
}
