import { describe, expect, it } from "vitest";

import {
  type AttendanceDay,
  dayKeyToIso,
  filterAttendanceDays,
  formatDayKey,
  monthRange,
  parseDayKey,
  usesFallback,
} from "@/features/export/domain/entities/attendance-export";

const DAY = 86_400_000;
const OCT_1 = Date.UTC(2026, 9, 1);

function day(
  offset: number,
  kind: AttendanceDay["kind"],
  start: number | null = null,
  end: number | null = null,
): AttendanceDay {
  return {
    day: OCT_1 + offset * DAY,
    kind,
    startMinutes: start,
    endMinutes: end,
  };
}

describe("filterAttendanceDays", () => {
  const days = [
    day(0, "office"),
    day(1, "remote"),
    day(5, "office"),
    day(40, "office"),
  ];

  it("keeps days inside the inclusive range with an enabled kind", () => {
    const out = filterAttendanceDays(days, {
      from: OCT_1,
      to: OCT_1 + 5 * DAY,
      kinds: new Set(["office"]),
    });
    expect(out.map((d) => d.day)).toEqual([OCT_1, OCT_1 + 5 * DAY]);
  });

  it("returns nothing when no kind is enabled", () => {
    expect(
      filterAttendanceDays(days, { from: 0, to: Infinity, kinds: new Set() }),
    ).toEqual([]);
  });
});

describe("usesFallback", () => {
  it("is true without a schedule or when end <= start", () => {
    expect(usesFallback(day(0, "office"))).toBe(true);
    expect(usesFallback(day(0, "office", 510, 510))).toBe(true);
    expect(usesFallback(day(0, "office", 510, 1020))).toBe(false);
  });

  it("is always true when the hours are applied to all days", () => {
    expect(usesFallback(day(0, "office", 510, 1020), true)).toBe(true);
  });
});

describe("monthRange", () => {
  it("spans the whole month, and the previous one with offset -1", () => {
    const date = new Date(2026, 9, 15);
    expect(monthRange(date)).toEqual({
      from: OCT_1,
      to: Date.UTC(2026, 9, 31),
    });
    expect(monthRange(date, -1)).toEqual({
      from: Date.UTC(2026, 8, 1),
      to: Date.UTC(2026, 8, 30),
    });
  });
});

describe("date and time inputs", () => {
  it("formats day keys as yyyy-MM-dd for file names", () => {
    expect(dayKeyToIso(OCT_1)).toBe("2026-10-01");
  });

  it("round-trips day keys through dd/mm/yyyy", () => {
    expect(formatDayKey(OCT_1)).toBe("01/10/2026");
    expect(parseDayKey("01/10/2026")).toBe(OCT_1);
    expect(parseDayKey(" 1/10/2026 ")).toBe(OCT_1);
    expect(parseDayKey("1.10.2026")).toBe(OCT_1);
  });

  it("rejects text that is not a real date", () => {
    expect(parseDayKey("")).toBeNull();
    expect(parseDayKey("31/02/2026")).toBeNull();
    expect(parseDayKey("10/13/2026")).toBeNull();
    expect(parseDayKey("2026-10-01")).toBeNull();
  });
});
