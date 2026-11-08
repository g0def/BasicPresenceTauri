import { describe, expect, it } from "vitest";

import {
  DEFAULT_SELECTION,
  effective,
  presentCategories,
  type BundleManifest,
  type BundleSelection,
} from "@/features/profile-transfer/domain/entities/bundle";

const manifest = (over: Partial<BundleManifest> = {}): BundleManifest => ({
  version: 1,
  compatible: true,
  exportedAt: 1_717_200_000_000,
  app: "0.2.0",
  profileFirstName: "Ada",
  profileLastName: "Lovelace",
  profileEnterprise: "Analytical Engine",
  days: 0,
  trips: 0,
  workEntries: 0,
  notes: 0,
  taskPresets: 0,
  commutes: 0,
  hasSettings: false,
  unknownModeIds: [],
  ...over,
});

describe("presentCategories", () => {
  it("returns only present categories in display order with their counts", () => {
    const result = presentCategories(
      manifest({
        days: 3,
        trips: 0,
        workEntries: 5,
        notes: 0,
        taskPresets: 2,
        commutes: 0,
        hasSettings: true,
      }),
    );

    // includeTrips, includeNotes, includeCommutes are zero -> omitted.
    expect(result).toEqual([
      { key: "includeDays", count: 3 },
      { key: "includeWorkHours", count: 5 },
      { key: "includeTaskPresets", count: 2 },
      { key: "includeSettings" },
    ]);
  });

  it("returns an empty list when the manifest contains nothing", () => {
    expect(presentCategories(manifest())).toEqual([]);
  });
});

describe("effective", () => {
  it("preserves the day-children when includeDays is true", () => {
    const sel: BundleSelection = { ...DEFAULT_SELECTION, includeDays: true };
    expect(effective(sel)).toEqual(sel);
  });

  it("forces the day-children to false when includeDays is false", () => {
    const sel: BundleSelection = {
      ...DEFAULT_SELECTION,
      includeDays: false,
      includeTrips: true,
      includeWorkHours: true,
      includeNotes: true,
    };

    expect(effective(sel)).toEqual({
      ...sel,
      includeTrips: false,
      includeWorkHours: false,
      includeNotes: false,
    });
  });

  it("leaves the non-child categories untouched when includeDays is false", () => {
    const sel: BundleSelection = {
      ...DEFAULT_SELECTION,
      includeDays: false,
      includeTaskPresets: true,
      includeCommutes: true,
      includeSettings: true,
    };

    const result = effective(sel);
    expect(result.includeTaskPresets).toBe(true);
    expect(result.includeCommutes).toBe(true);
    expect(result.includeSettings).toBe(true);
  });
});
