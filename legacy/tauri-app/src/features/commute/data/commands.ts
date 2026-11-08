// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.
import type {
  CommuteDto,
  EmissionFactorDto,
} from "@/features/commute/data/dto/commute.dto";
import type { CommuteSegment } from "@/entities/commute/model/commute";

interface CommuteFieldsArgs {
  name: string;
  roundTrip: boolean;
  segments: CommuteSegment[];
}

declare module "@/core/ipc" {
  interface CommandRegistry {
    list_emission_factors: { args: void; result: EmissionFactorDto[] };
    list_commutes: { args: { profileId: string }; result: CommuteDto[] };
    create_commute: {
      args: CommuteFieldsArgs & { profileId: string };
      result: CommuteDto;
    };
    update_commute: {
      args: CommuteFieldsArgs & { id: string };
      result: CommuteDto;
    };
    delete_commute: { args: { id: string }; result: void };
  }
}
