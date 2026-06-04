import type {
  Presence,
  SetPresenceInput,
} from "@/features/presence/domain/entities/presence";
import type { PresenceRepository } from "@/features/presence/domain/repositories/presence-repository";

export type SetPresenceUseCase = (input: SetPresenceInput) => Promise<Presence>;

export function makeSetPresenceUseCase(
  repo: PresenceRepository,
): SetPresenceUseCase {
  return (input) => repo.set(input);
}
