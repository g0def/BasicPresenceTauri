export interface Profile {
  id: string;
  firstName: string;
  lastName: string;
  enterprise: string;
  poste: string | null;
  createdAt: number;
  updatedAt: number;
}

/** Display name for a profile (`"Prénom Nom"`). Single source of truth so the
 * header badge and the settings list format names identically. */
export function fullName(p: Profile): string {
  return `${p.firstName} ${p.lastName}`;
}

/** Fields supplied when creating a profile (`createdAt`/`updatedAt` are backend-managed). */
export interface CreateProfileInput {
  firstName: string;
  lastName: string;
  enterprise: string;
  poste?: string | null;
}

/** A create payload plus the id of the profile being edited. */
export interface UpdateProfileInput extends CreateProfileInput {
  id: string;
}

/** The full profile set plus the active profile id (for the header switcher). */
export interface ProfileList {
  profiles: Profile[];
  activeProfileId: string | null;
}
