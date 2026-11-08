import { describe, expect, it } from "vitest";

import type { Co2ReferentialFactor } from "@/features/methodology/domain/entities/co2-referential";
import {
  factorsByCategory,
  workedExamples,
} from "@/features/methodology/domain/worked-examples";

const factor = (modeId: string, category: string, value: number) =>
  ({
    modeId,
    label: modeId,
    value,
    unit: "kgCO2e/km",
    category,
    isParam: false,
    scope: null,
    source: null,
    gridVariants: [],
  }) satisfies Co2ReferentialFactor;

describe("methodology worked examples", () => {
  it("computes every example from the live factors", () => {
    const examples = workedExamples({
      factorYear: 2026,
      radiativeForcing: 1.7,
      factors: [
        factor("car_petrol", "car", 0.2),
        factor("train_hs_fr", "rail", 0.003),
        factor("plane_short", "air", 0.25),
      ],
    });
    expect(examples).toEqual([
      { key: "car", km: 20, factor: 0.2, total: 8 },
      { key: "carpool", km: 20, factor: 0.2, total: 8 / 3 },
      { key: "train", km: 400, factor: 0.003, total: 2.4 },
      { key: "plane", km: 500, factor: 0.25, total: 125 },
    ]);
  });

  it("skips examples whose factor is missing", () => {
    expect(
      workedExamples({ factorYear: 2026, radiativeForcing: 1, factors: [] }),
    ).toEqual([]);
  });

  it("groups factors by category in order", () => {
    const grouped = factorsByCategory([
      factor("a", "car", 1),
      factor("b", "rail", 1),
      factor("c", "car", 1),
    ]);
    expect([...grouped.keys()]).toEqual(["car", "rail"]);
    expect(grouped.get("car")?.map((f) => f.modeId)).toEqual(["a", "c"]);
  });
});
