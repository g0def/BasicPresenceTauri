import type { Co2ReferentialDto } from "@/features/methodology/data/dto/co2-referential.dto";
import type { Co2Referential } from "@/features/methodology/domain/entities/co2-referential";

/** Map the wire DTO into the domain referential (carries every field, unlike the
 * trip-picker mapper which drops value/scope/source). */
export function toCo2Referential(dto: Co2ReferentialDto): Co2Referential {
  return {
    factorYear: dto.factorYear,
    radiativeForcing: dto.radiativeForcing,
    factors: dto.factors.map((f) => ({
      modeId: f.modeId,
      label: f.label,
      value: f.value,
      unit: f.unit,
      category: f.category,
      isParam: f.isParam,
      scope: f.scope ?? null,
      source: f.source ?? null,
      gridVariants: f.gridVariants ?? [],
    })),
  };
}
