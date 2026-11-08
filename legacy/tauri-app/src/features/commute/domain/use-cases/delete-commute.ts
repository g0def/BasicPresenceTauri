import type { CommuteRepository } from "@/features/commute/domain/repositories/commute-repository";

export type DeleteCommuteUseCase = (id: string) => Promise<void>;

export function makeDeleteCommuteUseCase(
  repo: CommuteRepository,
): DeleteCommuteUseCase {
  return (id) => repo.remove(id);
}
