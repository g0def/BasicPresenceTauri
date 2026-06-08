/** Transport-mode categories (mirror the backend referential; `other` is a
 * defensive fallback for unknown values). Drives icons and occupant visibility. */
export const COMMUTE_CATEGORIES = [
  "car",
  "active",
  "public_transport",
  "rail",
  "air",
  "other",
] as const;

export type CommuteCategory = (typeof COMMUTE_CATEGORIES)[number];

/** A selectable transport mode from the emission-factor referential. */
export interface EmissionFactor {
  /** The mode_id, e.g. `"car_petrol"`. */
  modeId: string;
  label: string;
  category: CommuteCategory;
  unit: string;
  isParam: boolean;
}

/** One leg of a commute. `distanceKm` is one-way; `occupants` only matters for
 * car-category modes (carpool). */
export interface CommuteSegment {
  modeId: string;
  distanceKm: number;
  occupants: number;
}

/** A saved, reusable named commute. `co2Kg` is an indicative per-trip footprint
 * annotated by the backend on list (absent on freshly created/updated rows). */
export interface Commute {
  id: string;
  profileId: string;
  name: string;
  roundTrip: boolean;
  segments: CommuteSegment[];
  co2Kg?: number;
  createdAt: number;
  updatedAt: number;
}

/** Fields supplied when creating a commute (profileId/timestamps backend-managed). */
export interface CreateCommuteInput {
  name: string;
  roundTrip: boolean;
  segments: CommuteSegment[];
}

/** A create payload plus the id of the commute being edited. */
export interface UpdateCommuteInput extends CreateCommuteInput {
  id: string;
}

/** One trip leg sent to `set_presence` when encoding an office/remote day. */
export interface TripInput {
  modeId: string;
  distanceKm: number;
  roundTrip: boolean;
  occupants: number;
}

/** Flatten a commute into the trip legs sent to the backend (the journey-level
 * round-trip flag is applied to every leg). */
export function commuteToTrips(
  c: Pick<Commute, "roundTrip" | "segments">,
): TripInput[] {
  return c.segments.map((s) => ({
    modeId: s.modeId,
    distanceKm: s.distanceKm,
    roundTrip: c.roundTrip,
    occupants: s.occupants,
  }));
}
