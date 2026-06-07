import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type { PresenceDto } from "@/features/presence/data/dto/presence.dto";
import { toPresence } from "@/features/presence/data/mappers/presence.mapper";
import type {
  ImportPresencesInput,
  ImportSummary,
  Presence,
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
    });
    return toPresence(dto);
  }

  async remove(id: string): Promise<void> {
    await invoke<void>(COMMANDS.deletePresence, { id });
  }

  async importMany(input: ImportPresencesInput): Promise<ImportSummary> {
    // The wire shape matches `ImportSummary`/`ImportPresenceEntry` 1:1 (camelCase),
    // so no DTO/mapper is needed. `undefined` timestamps are dropped by JSON
    // serialization and become `None` on the backend.
    return invoke<ImportSummary>(COMMANDS.importPresences, {
      profileId: input.profileId,
      entries: input.entries,
      replaceExisting: input.replaceExisting,
    });
  }
}
