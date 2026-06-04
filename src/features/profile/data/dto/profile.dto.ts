/** Wire shape returned by the Rust commands (serialized camelCase). */
export interface ProfileDto {
  id: string;
  firstName: string;
  lastName: string;
  enterprise: string;
  poste: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface ProfilesDto {
  profiles: ProfileDto[];
  activeProfileId: string | null;
}
