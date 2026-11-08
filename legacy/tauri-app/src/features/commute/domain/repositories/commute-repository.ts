import type {
  Commute,
  CreateCommuteInput,
  UpdateCommuteInput,
} from "@/entities/commute/model/commute";

/** Contract the presentation layer depends on; implemented in the data layer. */
export interface CommuteRepository {
  listByProfile(profileId: string): Promise<Commute[]>;
  create(profileId: string, input: CreateCommuteInput): Promise<Commute>;
  update(input: UpdateCommuteInput): Promise<Commute>;
  remove(id: string): Promise<void>;
}
