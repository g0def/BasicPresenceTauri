import { describe, expect, it } from "vitest";

import {
  toTaskPreset,
  toWorkEntry,
} from "@/features/work-hours/data/mappers/work-hours.mapper";

describe("work-hours mappers", () => {
  it("maps a TaskPresetDto to the domain entity", () => {
    expect(
      toTaskPreset({
        id: "p1",
        profileId: "profile-1",
        title: "Réunion",
        description: "Stand-up quotidien",
        defaultMinutes: 30,
        color: "#4B7F52",
        createdAt: 1,
        updatedAt: 2,
      }),
    ).toEqual({
      id: "p1",
      profileId: "profile-1",
      title: "Réunion",
      description: "Stand-up quotidien",
      defaultMinutes: 30,
      color: "#4B7F52",
      createdAt: 1,
      updatedAt: 2,
    });
  });

  it("maps a WorkEntryDto, keeping a null description", () => {
    expect(
      toWorkEntry({
        id: "e1",
        title: "Dev",
        description: null,
        minutes: 120,
        color: "#005377",
        position: 0,
      }),
    ).toEqual({
      id: "e1",
      title: "Dev",
      description: null,
      minutes: 120,
      color: "#005377",
      position: 0,
    });
  });
});
