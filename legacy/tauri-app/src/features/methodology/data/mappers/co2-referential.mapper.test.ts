import { describe, expect, it } from "vitest";

import type { Co2ReferentialDto } from "@/features/methodology/data/dto/co2-referential.dto";
import { toCo2Referential } from "@/features/methodology/data/mappers/co2-referential.mapper";

const dto: Co2ReferentialDto = {
  factorYear: 2026,
  radiativeForcing: 1.7,
  factors: [
    {
      modeId: "car_ev",
      label: "Voiture — électrique",
      value: 0.11,
      unit: "kgCO2e/veh.km",
      category: "car",
      isParam: true,
      gridVariants: [
        { country: "FR", value: 0.09 },
        { country: "BE", value: 0.11 },
      ],
      scope: "usage(mix réseau)+fabrication",
      source: "ADEME + mix réseau (Ember/OWID)",
    },
    {
      modeId: "office_day",
      label: "Jour au bureau (énergie bâtiment)",
      value: 3.5,
      unit: "kgCO2e/day",
      category: "building",
      isParam: false,
      gridVariants: [],
      scope: null,
      source: null,
    },
  ],
};

describe("co2-referential mapper", () => {
  it("maps year, radiative forcing and every factor field", () => {
    const r = toCo2Referential(dto);
    expect(r.factorYear).toBe(2026);
    expect(r.radiativeForcing).toBe(1.7);
    expect(r.factors).toHaveLength(2);

    const ev = r.factors[0];
    expect(ev.modeId).toBe("car_ev");
    expect(ev.value).toBe(0.11);
    expect(ev.scope).toBe("usage(mix réseau)+fabrication");
    expect(ev.source).toBe("ADEME + mix réseau (Ember/OWID)");
    expect(ev.gridVariants).toEqual([
      { country: "FR", value: 0.09 },
      { country: "BE", value: 0.11 },
    ]);
  });

  it("keeps the building factor and null scope/source", () => {
    const r = toCo2Referential(dto);
    const building = r.factors.find((f) => f.category === "building");
    expect(building?.modeId).toBe("office_day");
    expect(building?.scope).toBeNull();
    expect(building?.gridVariants).toEqual([]);
  });
});
