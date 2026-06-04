import { describe, expect, it } from "vitest";

import { toProfile } from "@/features/profile/data/mappers/profile.mapper";

describe("profile.mapper", () => {
  it("maps a profile DTO to a Profile entity", () => {
    expect(
      toProfile({
        id: "p1",
        firstName: "Ada",
        lastName: "Lovelace",
        enterprise: "Analytical Engine",
        poste: "Mathematician",
        createdAt: 1,
        updatedAt: 2,
      }),
    ).toEqual({
      id: "p1",
      firstName: "Ada",
      lastName: "Lovelace",
      enterprise: "Analytical Engine",
      poste: "Mathematician",
      createdAt: 1,
      updatedAt: 2,
    });
  });

  it("keeps a missing role as null", () => {
    const profile = toProfile({
      id: "p2",
      firstName: "Alan",
      lastName: "Turing",
      enterprise: "Bletchley",
      poste: null,
      createdAt: 1,
      updatedAt: 1,
    });

    expect(profile.poste).toBeNull();
  });
});
