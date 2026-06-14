import { describe, expect, it } from "vitest";

import { toPresence } from "@/features/presence/data/mappers/presence.mapper";

describe("presence.mapper", () => {
  it("maps a presence DTO to a Presence entity (keeping the `type` wire field)", () => {
    expect(
      toPresence({
        id: "x1",
        profileId: "p1",
        day: 1_717_200_000_000,
        type: "remote",
        co2Kg: 4.776,
        isEstimated: false,
        createdAt: 1,
        updatedAt: 2,
        workMinutes: 120,
      }),
    ).toEqual({
      id: "x1",
      profileId: "p1",
      day: 1_717_200_000_000,
      type: "remote",
      co2Kg: 4.776,
      isEstimated: false,
      createdAt: 1,
      updatedAt: 2,
      workMinutes: 120,
    });
  });

  it("preserves a null co2Kg (e.g. an imported day with no commute data)", () => {
    const mapped = toPresence({
      id: "x2",
      profileId: "p1",
      day: 1_717_200_000_000,
      type: "office",
      co2Kg: null,
      isEstimated: false,
      createdAt: 1,
      updatedAt: 2,
      workMinutes: 0,
    });
    expect(mapped.co2Kg).toBeNull();
    expect(mapped.workMinutes).toBe(0);
  });
});
