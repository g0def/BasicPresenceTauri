import type {
  Presence,
  SetPresenceInput,
} from "@/features/presence/domain/entities/presence";

/** Contract the presentation layer depends on; implemented in the data layer. */
export interface PresenceRepository {
  listByProfile(profileId: string): Promise<Presence[]>;
  /** Create or update the presence for a day; returns the persisted row. */
  set(input: SetPresenceInput): Promise<Presence>;
  remove(id: string): Promise<void>;
}
