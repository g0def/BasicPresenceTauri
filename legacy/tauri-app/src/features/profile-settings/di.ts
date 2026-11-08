// Ports this feature takes from the DI container (see core/di).
import type { ProfileSettingsRepository } from "@/features/profile-settings/domain/repositories/profile-settings-repository";

declare module "@/core/di" {
  interface Container {
    profileSettingsRepository: ProfileSettingsRepository;
  }
}
