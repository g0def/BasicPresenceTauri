import { describe, expect, it } from "vitest";

import { computeDonutArcs } from "@/features/work-hours/presentation/donut-geometry";

describe("computeDonutArcs", () => {
  it("returns nothing for an empty day", () => {
    expect(computeDonutArcs([])).toEqual([]);
    expect(computeDonutArcs([{ id: "a", minutes: 0 }])).toEqual([]);
  });

  it("closes the full ring for a single task (no gap)", () => {
    const [arc] = computeDonutArcs([{ id: "a", minutes: 90 }]);

    expect(arc.start).toBe(0);
    expect(arc.length).toBe(100);
  });

  it("splits the ring proportionally, counter-clockwise from 12 o'clock", () => {
    const arcs = computeDonutArcs([
      { id: "a", minutes: 120 }, // 50%
      { id: "b", minutes: 60 }, // 25%
      { id: "c", minutes: 60 }, // 25%
    ]);

    // `start` is in clockwise path units: the first task ends just before 12
    // o'clock (so it OCCUPIES the top-left), the next ones follow leftward.
    expect(arcs[0].start).toBeCloseTo(50.75);
    expect(arcs[0].length).toBeCloseTo(48.5);
    expect(arcs[1].start).toBeCloseTo(25.75);
    expect(arcs[1].length).toBeCloseTo(23.5);
    expect(arcs[2].start).toBeCloseTo(0.75);
    expect(arcs[2].length).toBeCloseTo(23.5);

    // Consecutive tasks stay adjacent: each arc ends where the previous starts.
    expect(arcs[1].start + arcs[1].length + 1.5).toBeCloseTo(arcs[0].start);
    expect(arcs[2].start + arcs[2].length + 1.5).toBeCloseTo(arcs[1].start);
  });

  it("keeps a tiny task visible via the minimum arc length", () => {
    const arcs = computeDonutArcs([
      { id: "tiny", minutes: 5 },
      { id: "big", minutes: 480 },
    ]);

    expect(arcs[0].length).toBeGreaterThanOrEqual(0.5);
  });
});
