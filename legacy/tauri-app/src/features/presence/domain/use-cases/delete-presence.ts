import type { PresenceRepository } from "@/features/presence/domain/repositories/presence-repository";

export type DeletePresenceUseCase = (id: string) => Promise<void>;

export function makeDeletePresenceUseCase(
  repo: PresenceRepository,
): DeletePresenceUseCase {
  return (id) => repo.remove(id);
}
