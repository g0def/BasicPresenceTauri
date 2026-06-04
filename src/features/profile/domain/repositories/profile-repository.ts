import type {
  CreateProfileInput,
  Profile,
  ProfileList,
  UpdateProfileInput,
} from "@/features/profile/domain/entities/profile";

/** Contract the presentation layer depends on; implemented in the data layer. */
export interface ProfileRepository {
  list(): Promise<ProfileList>;
  create(input: CreateProfileInput): Promise<Profile>;
  update(input: UpdateProfileInput): Promise<Profile>;
  remove(id: string): Promise<void>;
  setActive(id: string): Promise<void>;
}
