import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import { toProfile } from "@/features/profile/data/mappers/profile.mapper";
import type {
  CreateProfileInput,
  Profile,
  ProfileList,
  UpdateProfileInput,
} from "@/entities/profile/model/profile";
import type { ProfileRepository } from "@/features/profile/domain/repositories/profile-repository";

/** ProfileRepository implementation backed by Tauri IPC commands. */
export class TauriProfileRepository implements ProfileRepository {
  async list(): Promise<ProfileList> {
    const dto = await invoke(COMMANDS.listProfiles);
    return {
      profiles: dto.profiles.map(toProfile),
      activeProfileId: dto.activeProfileId,
    };
  }

  async create(input: CreateProfileInput): Promise<Profile> {
    const dto = await invoke(COMMANDS.createProfile, {
      firstName: input.firstName,
      lastName: input.lastName,
      enterprise: input.enterprise,
      poste: input.poste ?? null,
    });
    return toProfile(dto);
  }

  async update(input: UpdateProfileInput): Promise<Profile> {
    const dto = await invoke(COMMANDS.updateProfile, {
      id: input.id,
      firstName: input.firstName,
      lastName: input.lastName,
      enterprise: input.enterprise,
      poste: input.poste ?? null,
    });
    return toProfile(dto);
  }

  async remove(id: string): Promise<void> {
    await invoke(COMMANDS.deleteProfile, { id });
  }

  async setActive(id: string): Promise<void> {
    await invoke(COMMANDS.setActiveProfile, { id });
  }
}
