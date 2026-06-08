import type { EmissionFactor } from "@/features/commute/domain/entities/commute";

/** Read-only access to the emission-factor referential (for the mode pickers). */
export interface EmissionFactorRepository {
  list(): Promise<EmissionFactor[]>;
}
