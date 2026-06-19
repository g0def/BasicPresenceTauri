import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type {
  PresenceDto,
  PresenceTripDto,
} from "@/features/presence/data/dto/presence.dto";
import {
  toPresence,
  toPresenceTrip,
} from "@/features/presence/data/mappers/presence.mapper";
import type {
  Presence,
  PresenceTrip,
  SetPresenceInput,
} from "@/features/presence/domain/entities/presence";
import type { PresenceRepository } from "@/features/presence/domain/repositories/presence-repository";

/** PresenceRepository implementation backed by Tauri IPC commands. */
export class TauriPresenceRepository implements PresenceRepository {
  async listByProfile(profileId: string): Promise<Presence[]> {
    const dtos = await invoke<PresenceDto[]>(COMMANDS.listPresences, {
      profileId,
    });
    return dtos.map(toPresence);
  }

  async set(input: SetPresenceInput): Promise<Presence> {
    const dto = await invoke<PresenceDto>(COMMANDS.setPresence, {
      profileId: input.profileId,
      day: input.day,
      type: input.type,
      trips: input.trips ?? [],
    });
    return toPresence(dto);
  }

  async remove(id: string): Promise<void> {
    await invoke<void>(COMMANDS.deletePresence, { id });
  }

  async getTrips(presenceId: string): Promise<PresenceTrip[]> {
    const dtos = await invoke<PresenceTripDto[]>(COMMANDS.getPresenceTrips, {
      presenceId,
    });
    return dtos.map(toPresenceTrip);
  }
}
