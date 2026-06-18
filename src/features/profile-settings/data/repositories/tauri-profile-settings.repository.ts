import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type { ProfileSettingsDto } from "@/features/profile-settings/data/dto/profile-settings.dto";
import {
  toProfileSettings,
  toProfileSettingsDto,
} from "@/features/profile-settings/data/mappers/profile-settings.mapper";
import type { ProfileSettings } from "@/features/profile-settings/domain/entities/profile-settings";
import type { ProfileSettingsRepository } from "@/features/profile-settings/domain/repositories/profile-settings-repository";

/** ProfileSettingsRepository implementation backed by Tauri IPC commands. */
export class TauriProfileSettingsRepository implements ProfileSettingsRepository {
  async get(profileId: string): Promise<ProfileSettings> {
    const dto = await invoke<ProfileSettingsDto>(COMMANDS.getProfileSettings, {
      profileId,
    });
    return toProfileSettings(dto);
  }

  async set(
    profileId: string,
    settings: ProfileSettings,
  ): Promise<ProfileSettings> {
    const dto = await invoke<ProfileSettingsDto>(COMMANDS.setProfileSettings, {
      profileId,
      settings: toProfileSettingsDto(settings),
    });
    return toProfileSettings(dto);
  }
}
