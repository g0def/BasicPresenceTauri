import type { PresenceType } from "@/features/presence/domain/entities/presence";

/** Wire shape returned by the Rust commands (serialized camelCase). */
export interface PresenceDto {
  id: string;
  profileId: string;
  day: number;
  type: PresenceType;
  co2Kg: number | null;
  isEstimated: boolean;
  createdAt: number;
  updatedAt: number;
}

/** A persisted trip snapshot returned by `get_presence_trips`. */
export interface PresenceTripDto {
  id: string;
  modeId: string;
  distanceKm: number;
  roundTrip: boolean;
  occupants: number;
  co2Kg: number;
  isEstimated: boolean;
  position: number;
}
