import type { PresenceType } from "@/features/presence/domain/entities/presence";

/** Wire shape returned by the Rust commands (serialized camelCase). */
export interface PresenceDto {
  id: string;
  profileId: string;
  day: number;
  type: PresenceType;
  createdAt: number;
  updatedAt: number;
}
