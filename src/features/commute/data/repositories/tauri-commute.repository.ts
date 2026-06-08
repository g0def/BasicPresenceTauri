import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type { CommuteDto } from "@/features/commute/data/dto/commute.dto";
import { toCommute } from "@/features/commute/data/mappers/commute.mapper";
import type {
  Commute,
  CreateCommuteInput,
  UpdateCommuteInput,
} from "@/features/commute/domain/entities/commute";
import type { CommuteRepository } from "@/features/commute/domain/repositories/commute-repository";

/** CommuteRepository implementation backed by Tauri IPC commands. */
export class TauriCommuteRepository implements CommuteRepository {
  async listByProfile(profileId: string): Promise<Commute[]> {
    const dtos = await invoke<CommuteDto[]>(COMMANDS.listCommutes, {
      profileId,
    });
    return dtos.map(toCommute);
  }

  async create(profileId: string, input: CreateCommuteInput): Promise<Commute> {
    const dto = await invoke<CommuteDto>(COMMANDS.createCommute, {
      profileId,
      name: input.name,
      roundTrip: input.roundTrip,
      segments: input.segments,
    });
    return toCommute(dto);
  }

  async update(input: UpdateCommuteInput): Promise<Commute> {
    const dto = await invoke<CommuteDto>(COMMANDS.updateCommute, {
      id: input.id,
      name: input.name,
      roundTrip: input.roundTrip,
      segments: input.segments,
    });
    return toCommute(dto);
  }

  async remove(id: string): Promise<void> {
    await invoke<void>(COMMANDS.deleteCommute, { id });
  }
}
