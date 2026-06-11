import { describe, expect, it } from "vitest";

import {
  formatClockTime,
  formatMinutes,
  parseClockTime,
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

describe("parseClockTime", () => {
  it("accepts a bare hour", () => {
    expect(parseClockTime("8")).toBe(480);
    expect(parseClockTime("08")).toBe(480);
    expect(parseClockTime("0")).toBe(0);
    expect(parseClockTime("23")).toBe(1380);
  });

  it("accepts the usual hour/minute separators", () => {
    expect(parseClockTime("8:30")).toBe(510);
    expect(parseClockTime("8h30")).toBe(510);
    expect(parseClockTime("8H30")).toBe(510);
    expect(parseClockTime("8.30")).toBe(510);
    expect(parseClockTime("8,30")).toBe(510);
    expect(parseClockTime(" 8 : 30 ")).toBe(510);
  });

  it("accepts compact digit-only forms", () => {
    expect(parseClockTime("830")).toBe(510);
    expect(parseClockTime("0830")).toBe(510);
    expect(parseClockTime("1745")).toBe(1065);
  });

  it("treats a trailing separator as a round hour", () => {
    expect(parseClockTime("8h")).toBe(480);
    expect(parseClockTime("8:")).toBe(480);
  });

  it("rejects out-of-range and malformed input", () => {
    expect(parseClockTime("")).toBeNull();
    expect(parseClockTime("24")).toBeNull();
    expect(parseClockTime("8:60")).toBeNull();
    expect(parseClockTime("875")).toBeNull();
    expect(parseClockTime("abc")).toBeNull();
    expect(parseClockTime("8:30:00")).toBeNull();
  });
});
