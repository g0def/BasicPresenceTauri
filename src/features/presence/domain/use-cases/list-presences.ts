import type { Presence } from "@/features/presence/domain/entities/presence";
import type { PresenceRepository } from "@/features/presence/domain/repositories/presence-repository";

export type ListPresencesUseCase = (profileId: string) => Promise<Presence[]>;

export function makeListPresencesUseCase(
  repo: PresenceRepository,
): ListPresencesUseCase {
  return (profileId) => repo.listByProfile(profileId);
}
