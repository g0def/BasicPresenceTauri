import type { EmissionFactor } from "@/entities/commute/model/commute";

/** Read-only access to the emission-factor referential (for the mode pickers). */
export interface EmissionFactorRepository {
  list(): Promise<EmissionFactor[]>;
}
