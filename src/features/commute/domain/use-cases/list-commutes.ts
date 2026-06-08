import type { Commute } from "@/features/commute/domain/entities/commute";
import type { CommuteRepository } from "@/features/commute/domain/repositories/commute-repository";

export type ListCommutesUseCase = (profileId: string) => Promise<Commute[]>;

export function makeListCommutesUseCase(
  repo: CommuteRepository,
): ListCommutesUseCase {
  return (profileId) => repo.listByProfile(profileId);
}
