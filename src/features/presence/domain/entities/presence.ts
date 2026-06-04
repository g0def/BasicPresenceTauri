/** The four presence types, in display order. */
export const PRESENCE_TYPES = [
  "office",
  "remote",
  "vacation",
  "holiday",
] as const;

export type PresenceType = (typeof PRESENCE_TYPES)[number];

export interface Presence {
  id: string;
  profileId: string;
  /** Epoch ms at UTC midnight of the day concerned. */
  day: number;
  type: PresenceType;
  createdAt: number;
  updatedAt: number;
}

/** Fields supplied when setting a presence (`id`/timestamps are backend-managed). */
export interface SetPresenceInput {
  profileId: string;
  day: number;
  type: PresenceType;
}
