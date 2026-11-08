// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.
import type {
  PresenceDto,
  PresenceTripDto,
} from "@/features/presence/data/dto/presence.dto";
import type {
  PresenceTrip,
  PresenceType,
} from "@/entities/presence/model/presence";

declare module "@/core/ipc" {
  interface CommandRegistry {
    list_presences: { args: { profileId: string }; result: PresenceDto[] };
    set_presence: {
      args: {
        profileId: string;
        day: number;
        type: PresenceType;
        trips: PresenceTrip[];
      };
      result: PresenceDto;
    };
    delete_presence: { args: { id: string }; result: void };
    get_presence_trips: {
      args: { presenceId: string };
      result: PresenceTripDto[];
    };
  }
}
