import type {
  Commute,
  UpdateCommuteInput,
} from "@/entities/commute/model/commute";
import type { CommuteRepository } from "@/features/commute/domain/repositories/commute-repository";

export type UpdateCommuteUseCase = (
  input: UpdateCommuteInput,
) => Promise<Commute>;

export function makeUpdateCommuteUseCase(
  repo: CommuteRepository,
): UpdateCommuteUseCase {
  return (input) => repo.update(input);
}
