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
        createdAt: 1,
        updatedAt: 2,
      }),
    ).toEqual({
      id: "x1",
      profileId: "p1",
      day: 1_717_200_000_000,
      type: "remote",
      createdAt: 1,
      updatedAt: 2,
    });
  });
});
