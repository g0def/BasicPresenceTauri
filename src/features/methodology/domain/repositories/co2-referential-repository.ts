import type { Co2Referential } from "@/features/methodology/domain/entities/co2-referential";

export interface Co2ReferentialRepository {
  /** The full CO2 referential (factors + sources), for the methodology page. */
  get(): Promise<Co2Referential>;
}
