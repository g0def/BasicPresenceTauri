import { describe, expect, it } from "vitest";

import { dayKey } from "@/features/presence/presentation/day-key";

describe("dayKey", () => {
  it("returns the UTC-midnight epoch ms for the date's calendar day", () => {
    // A local date with a wall-clock time maps to UTC midnight of that Y/M/D.
    expect(dayKey(new Date(2024, 5, 1, 13, 45))).toBe(Date.UTC(2024, 5, 1));
  });

  it("is stable across times within the same calendar day", () => {
    const morning = dayKey(new Date(2024, 0, 15, 1, 0));
    const night = dayKey(new Date(2024, 0, 15, 23, 59));

    expect(morning).toBe(night);
    // A canonical day key is always a whole number of UTC days.
    expect(morning % 86_400_000).toBe(0);
  });
});
