import { describe, expect, it } from "vitest";

import type {
  Presence,
  PresenceType,
} from "@/features/presence/domain/entities/presence";
import {
  breakdownByType,
  bucketPresences,
  periodKey,
} from "@/features/stats/domain/aggregate";

/** Minimal presence builder — only the fields the aggregation reads matter. */
function presence(
  day: number,
  overrides: Partial<Presence> & { type?: PresenceType } = {},
): Presence {
  return {
    id: `p-${day}-${overrides.type ?? "office"}`,
    profileId: "profile",
    day,
    type: "office",
    co2Kg: 0,
    isEstimated: false,
    createdAt: 0,
    updatedAt: 0,
    workMinutes: 0,
    ...overrides,
  };
}

const utc = (y: number, m: number, d: number) => Date.UTC(y, m, d);

describe("bucketPresences", () => {
  it("returns nothing for an empty dataset", () => {
    expect(bucketPresences([], "month")).toEqual([]);
    expect(bucketPresences([], "week")).toEqual([]);
  });

  it("collapses same-month days into one bucket and sums metrics", () => {
    const buckets = bucketPresences(
      [
        presence(utc(2026, 2, 3), { co2Kg: 4, workMinutes: 480 }),
        presence(utc(2026, 2, 18), { co2Kg: 6, workMinutes: 60 }),
      ],
      "month",
    );

    expect(buckets).toHaveLength(1);
    expect(buckets[0]).toMatchObject({ co2Kg: 10, workMinutes: 540, dayCount: 2 });
  });

  it("gap-fills empty months between the first and last bucket", () => {
    const buckets = bucketPresences(
      [
        presence(utc(2026, 0, 10), { workMinutes: 60 }),
        presence(utc(2026, 2, 10), { workMinutes: 60 }),
      ],
      "month",
    );

    // Jan, Feb (filled zero), Mar.
    expect(buckets).toHaveLength(3);
    expect(buckets.map((b) => b.dayCount)).toEqual([1, 0, 1]);
    expect(buckets[1]).toMatchObject({ co2Kg: 0, workMinutes: 0, dayCount: 0 });
  });

  it("extends the timeline up to `until` even with no recent data", () => {
    // Data only in January, but `until` is in March → the current (empty)
    // months stay on the chart so the latest period is always visible.
    const buckets = bucketPresences(
      [presence(utc(2026, 0, 10))],
      "month",
      utc(2026, 2, 5),
    );

    expect(buckets).toHaveLength(3);
    expect(buckets[buckets.length - 1].dayCount).toBe(0);
  });

  it("excludes null co2Kg from the CO2 sum but still counts the day", () => {
    const buckets = bucketPresences(
      [
        presence(utc(2026, 2, 3), { type: "office", co2Kg: 5, workMinutes: 480 }),
        presence(utc(2026, 2, 4), {
          type: "vacation",
          co2Kg: null,
          workMinutes: 0,
        }),
      ],
      "month",
    );

    expect(buckets).toHaveLength(1);
    expect(buckets[0]).toMatchObject({ co2Kg: 5, dayCount: 2 });
  });

  it("buckets ISO weeks by ISO week-year across the New Year boundary", () => {
    // 2025-12-29 (Mon) and 2026-01-01 (Thu) both fall in ISO week 1 of 2026.
    const buckets = bucketPresences(
      [presence(utc(2025, 11, 29)), presence(utc(2026, 0, 1))],
      "week",
    );

    expect(buckets).toHaveLength(1);
    expect(buckets[0].key).toBe(2026 * 100 + 1);
    expect(buckets[0].dayCount).toBe(2);
  });

  it("separates distinct ISO weeks", () => {
    // ISO week 1 (2025-12-29) vs ISO week 2 (2026-01-05, the next Monday).
    const buckets = bucketPresences(
      [presence(utc(2025, 11, 29)), presence(utc(2026, 0, 5))],
      "week",
    );

    expect(buckets.map((b) => b.key)).toEqual([2026 * 100 + 1, 2026 * 100 + 2]);
    expect(buckets.map((b) => b.dayCount)).toEqual([1, 1]);
  });
});

describe("periodKey", () => {
  it("matches the bucket key its presence lands in (month)", () => {
    const day = utc(2026, 2, 18);
    const [bucket] = bucketPresences([presence(day)], "month");
    expect(periodKey(day, "month")).toBe(bucket.key);
  });

  it("matches the bucket key its presence lands in (week, year boundary)", () => {
    const day = utc(2025, 11, 29); // ISO week 1 of 2026
    const [bucket] = bucketPresences([presence(day)], "week");
    expect(periodKey(day, "week")).toBe(bucket.key);
    expect(periodKey(day, "week")).toBe(2026 * 100 + 1);
  });
});

describe("breakdownByType", () => {
  const all = new Set<PresenceType>(["office", "remote", "vacation", "holiday"]);

  it("counts days per type in canonical order", () => {
    const slices = breakdownByType(
      [
        presence(utc(2026, 0, 1), { type: "remote" }),
        presence(utc(2026, 0, 2), { type: "office" }),
        presence(utc(2026, 0, 3), { type: "remote" }),
      ],
      all,
    );

    // office before remote (PRESENCE_TYPES order), regardless of input order.
    expect(slices).toEqual([
      { type: "office", days: 1 },
      { type: "remote", days: 2 },
    ]);
  });

  it("drops types excluded by the filter", () => {
    const slices = breakdownByType(
      [
        presence(utc(2026, 0, 1), { type: "office" }),
        presence(utc(2026, 0, 2), { type: "vacation" }),
      ],
      new Set<PresenceType>(["office"]),
    );

    expect(slices).toEqual([{ type: "office", days: 1 }]);
  });
});
