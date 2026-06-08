import type { PresenceTrip } from "@/features/presence/domain/entities/presence";
import type { PresenceRepository } from "@/features/presence/domain/repositories/presence-repository";

export type GetPresenceTripsUseCase = (
  presenceId: string,
) => Promise<PresenceTrip[]>;

export function makeGetPresenceTripsUseCase(
  repo: PresenceRepository,
): GetPresenceTripsUseCase {
  return (presenceId) => repo.getTrips(presenceId);
}
