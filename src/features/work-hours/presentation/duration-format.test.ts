import { describe, expect, it } from "vitest";

import {
  formatClockTime,
  formatMinutes,
} from "@/features/work-hours/presentation/duration-format";

describe("formatMinutes", () => {
  it("formats sub-hour durations in minutes", () => {
    expect(formatMinutes(5)).toBe("5min");
    expect(formatMinutes(45)).toBe("45min");
  });

  it("formats whole hours without minutes", () => {
    expect(formatMinutes(60)).toBe("1h");
    expect(formatMinutes(480)).toBe("8h");
  });

  it("zero-pads minutes past the hour", () => {
    expect(formatMinutes(65)).toBe("1h05");
    expect(formatMinutes(450)).toBe("7h30");
  });

  it("clamps negatives to zero", () => {
    expect(formatMinutes(-10)).toBe("0min");
    expect(formatMinutes(0)).toBe("0min");
  });
});

describe("formatClockTime", () => {
  it("renders HH:MM with zero padding", () => {
    expect(formatClockTime(0)).toBe("00:00");
    expect(formatClockTime(510)).toBe("08:30");
    expect(formatClockTime(1020)).toBe("17:00");
  });

  it("wraps a stacked end time past midnight", () => {
    // 23:30 + 1h of tasks ends at 00:30 the next day.
    expect(formatClockTime(1470)).toBe("00:30");
  });
});
