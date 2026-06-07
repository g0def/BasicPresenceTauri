import { describe, expect, it } from "vitest";

import {
  findConflicts,
  parseOldExport,
} from "@/features/presence/data/import/parse-old-export";

describe("parseOldExport", () => {
  it("maps a calendar date to UTC-midnight epoch ms (timezone-independent)", () => {
    const { entries, invalid } = parseOldExport([
      {
        date: "2026-12-25T00:00:00",
        mode: "Holiday",
        createdAt: null,
        modifiedAt: null,
      },
    ]);
    expect(invalid).toHaveLength(0);
    expect(entries).toHaveLength(1);
    expect(entries[0].day).toBe(Date.UTC(2026, 11, 25));
    expect(entries[0].type).toBe("holiday");
  });

  it("normalizes the mode case", () => {
    const { entries } = parseOldExport([
      { date: "2026-06-05", mode: "REMOTE" },
      { date: "2026-06-04", mode: "office" },
      { date: "2026-06-03", mode: "Vacation" },
    ]);
    expect(entries.map((e) => e.type)).toEqual([
      "remote",
      "office",
      "vacation",
    ]);
  });

  it("parses createdAt with its offset and treats modifiedAt:null as undefined", () => {
    const { entries } = parseOldExport([
      {
        date: "2026-05-20T00:00:00",
        mode: "Remote",
        createdAt: "2026-05-20T16:10:55.23+02:00",
        modifiedAt: null,
      },
    ]);
    expect(entries[0].createdAt).toBe(
      Date.parse("2026-05-20T16:10:55.23+02:00"),
    );
    expect(entries[0].updatedAt).toBeUndefined();
  });

  it("leaves timestamps undefined when absent", () => {
    const { entries } = parseOldExport([
      { date: "2026-01-01", mode: "Holiday" },
    ]);
    expect(entries[0].createdAt).toBeUndefined();
    expect(entries[0].updatedAt).toBeUndefined();
  });

  it("collects invalid rows without aborting the valid ones", () => {
    const { entries, invalid } = parseOldExport([
      { date: "2026-06-05", mode: "Office" },
      { date: "2026-06-04", mode: "Carpool" }, // unknown mode
      { date: "not-a-date", mode: "Remote" }, // bad date
      { mode: "Remote" }, // missing date
      { date: "2026-06-02" }, // missing mode
    ]);
    expect(entries).toHaveLength(1);
    expect(invalid.map((r) => r.reason.code)).toEqual([
      "unknownMode",
      "badDate",
      "missingDate",
      "missingMode",
    ]);
    // The offending value / raw date are carried so the UI can show them.
    const unknown = invalid.find((r) => r.reason.code === "unknownMode");
    expect(unknown?.reason).toMatchObject({ value: "Carpool" });
    expect(unknown?.date).toBe("2026-06-04");
  });

  it("rejects an impossible calendar date", () => {
    const { entries, invalid } = parseOldExport([
      { date: "2026-02-31", mode: "Office" },
    ]);
    expect(entries).toHaveLength(0);
    expect(invalid[0].reason.code).toBe("badDate");
  });

  it("flags a non-array document as fatal", () => {
    const result = parseOldExport({ not: "an array" });
    expect(result.fatal).toBe("notArray");
    expect(result.entries).toHaveLength(0);
  });

  it("treats an empty array as no entries and no invalid rows", () => {
    const result = parseOldExport([]);
    expect(result.entries).toHaveLength(0);
    expect(result.invalid).toHaveLength(0);
    expect(result.fatal).toBeUndefined();
  });
});

describe("findConflicts", () => {
  it("returns the unique days that already exist", () => {
    const d1 = Date.UTC(2026, 5, 5);
    const d2 = Date.UTC(2026, 5, 6);
    const conflicts = findConflicts(
      [
        { day: d1, type: "office" },
        { day: d1, type: "remote" }, // duplicate of d1 within the file
        { day: d2, type: "office" },
      ],
      new Set([d1]),
    );
    expect(conflicts).toEqual([d1]);
  });
});
