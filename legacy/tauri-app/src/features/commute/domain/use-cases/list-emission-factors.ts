import type { EmissionFactor } from "@/entities/commute/model/commute";
import type { EmissionFactorRepository } from "@/features/commute/domain/repositories/emission-factor-repository";

export type ListEmissionFactorsUseCase = () => Promise<EmissionFactor[]>;

export function makeListEmissionFactorsUseCase(
  repo: EmissionFactorRepository,
): ListEmissionFactorsUseCase {
  return () => repo.list();
}
