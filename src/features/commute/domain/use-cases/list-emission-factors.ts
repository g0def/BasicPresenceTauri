import type { EmissionFactor } from "@/features/commute/domain/entities/commute";
import type { EmissionFactorRepository } from "@/features/commute/domain/repositories/emission-factor-repository";

export type ListEmissionFactorsUseCase = () => Promise<EmissionFactor[]>;

export function makeListEmissionFactorsUseCase(
  repo: EmissionFactorRepository,
): ListEmissionFactorsUseCase {
  return () => repo.list();
}
