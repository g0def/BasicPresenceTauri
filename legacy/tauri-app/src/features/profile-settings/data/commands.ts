// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.
import type { ProfileSettingsDto } from "@/features/profile-settings/data/dto/profile-settings.dto";

declare module "@/core/ipc" {
  interface CommandRegistry {
    get_profile_settings: {
      args: { profileId: string };
      result: ProfileSettingsDto;
    };
    set_profile_settings: {
      args: { profileId: string; settings: ProfileSettingsDto };
      result: ProfileSettingsDto;
    };
  }
}
