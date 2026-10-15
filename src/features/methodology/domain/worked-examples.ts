import type {
  Co2Referential,
  Co2ReferentialFactor,
} from "@/features/methodology/domain/entities/co2-referential";

/** One illustrative calculation shown on the methodology page. */
export interface WorkedExample {
  key: "car" | "carpool" | "train" | "plane";
  /** One-way distance, km. */
  km: number;
  /** Emission factor used, kgCO2e per km. */
  factor: number;
  /** Resulting footprint, kgCO2e. */
  total: number;
}

/** Group the referential's factors by category, preserving their order. */
export function factorsByCategory(
  factors: Co2ReferentialFactor[],
): Map<string, Co2ReferentialFactor[]> {
  const map = new Map<string, Co2ReferentialFactor[]>();
  for (const f of factors) {
    const list = map.get(f.category) ?? [];
    list.push(f);
    map.set(f.category, list);
  }
  return map;
}

/**
 * Worked examples computed from the LIVE factors (never hard-coded results):
 * a 20 km car commute (alone, then shared by 3), a 400 km high-speed train
 * round trip and a 500 km one-way short-haul flight. Examples whose factor is
 * missing from the referential are skipped.
 */
export function workedExamples(referential: Co2Referential): WorkedExample[] {
  const byId = new Map(referential.factors.map((f) => [f.modeId, f]));
  const out: WorkedExample[] = [];
  const carPetrol = byId.get("car_petrol");
  if (carPetrol) {
    const roundTrip = 20 * 2 * carPetrol.value;
    out.push({ key: "car", km: 20, factor: carPetrol.value, total: roundTrip });
    out.push({
      key: "carpool",
      km: 20,
      factor: carPetrol.value,
      total: roundTrip / 3,
    });
  }
  const tgv = byId.get("train_hs_fr");
  if (tgv) {
    out.push({
      key: "train",
      km: 400,
      factor: tgv.value,
      total: 400 * 2 * tgv.value,
    });
  }
  const plane = byId.get("plane_short");
  if (plane) {
    out.push({
      key: "plane",
      km: 500,
      factor: plane.value,
      total: 500 * plane.value,
    });
  }
  return out;
}
