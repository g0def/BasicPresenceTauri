// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.
import type {
  ProfileDto,
  ProfilesDto,
} from "@/features/profile/data/dto/profile.dto";

interface ProfileFieldsArgs {
  firstName: string;
  lastName: string;
  enterprise: string;
  poste: string | null;
}

declare module "@/core/ipc" {
  interface CommandRegistry {
    list_profiles: { args: void; result: ProfilesDto };
    create_profile: { args: ProfileFieldsArgs; result: ProfileDto };
    update_profile: {
      args: ProfileFieldsArgs & { id: string };
      result: ProfileDto;
    };
    delete_profile: { args: { id: string }; result: void };
    set_active_profile: { args: { id: string }; result: void };
  }
}
