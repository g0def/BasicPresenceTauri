import type {
  Commute,
  CreateCommuteInput,
} from "@/features/commute/domain/entities/commute";
import type { CommuteRepository } from "@/features/commute/domain/repositories/commute-repository";

export type CreateCommuteUseCase = (
  profileId: string,
  input: CreateCommuteInput,
) => Promise<Commute>;

export function makeCreateCommuteUseCase(
  repo: CommuteRepository,
): CreateCommuteUseCase {
  return (profileId, input) => repo.create(profileId, input);
}
