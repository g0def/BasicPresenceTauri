import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AppError } from "@/core/errors";
import type {
  AttendanceDay,
  AttendanceExportRequest,
} from "@/features/export/domain/entities/attendance-export";
import { useAttendanceExport } from "@/features/export/presentation/hooks/use-attendance-export";
import { withContainer } from "@/test/app-providers";

const listAttendanceDays = vi.fn();
const exportAttendance = vi.fn();
const wrapper = withContainer({
  exportRepository: {
    exportProfileData: vi.fn(),
    listAttendanceDays,
    exportAttendance,
  },
});

const DAYS: AttendanceDay[] = [
  { day: 0, kind: "office", startMinutes: 480, endMinutes: 1020 },
];
const REQUEST: AttendanceExportRequest = {
  employeeName: "Ada Lovelace",
  days: [0],
  fallbackStartMinutes: 480,
  fallbackEndMinutes: 1020,
  applyToAll: false,
};

afterEach(() => vi.clearAllMocks());

describe("useAttendanceExport", () => {
  it("loads the days only when enabled", async () => {
    listAttendanceDays.mockResolvedValue(DAYS);
    const { result, rerender } = renderHook(
      ({ enabled }) => useAttendanceExport("p1", enabled),
      { wrapper, initialProps: { enabled: false } },
    );
    expect(listAttendanceDays).not.toHaveBeenCalled();
    rerender({ enabled: true });
    await waitFor(() => expect(result.current.days).toEqual(DAYS));
    expect(listAttendanceDays).toHaveBeenCalledWith("p1");
  });

  it("returns the summary of a successful export", async () => {
    listAttendanceDays.mockResolvedValue(DAYS);
    const summary = { rows: 1, fallbackRows: 0, path: "/tmp/odoo.csv" };
    exportAttendance.mockResolvedValue(summary);
    const { result } = renderHook(() => useAttendanceExport("p1", true), {
      wrapper,
    });
    let out: unknown;
    await act(async () => {
      out = await result.current.exportAttendance(REQUEST, "odoo.csv");
    });
    expect(out).toEqual(summary);
    expect(exportAttendance).toHaveBeenCalledWith("p1", REQUEST, "odoo.csv");
    expect(result.current.error).toBeNull();
  });

  it("returns null without error when the dialog is cancelled", async () => {
    listAttendanceDays.mockResolvedValue(DAYS);
    exportAttendance.mockResolvedValue(null);
    const { result } = renderHook(() => useAttendanceExport("p1", true), {
      wrapper,
    });
    let out: unknown = "unset";
    await act(async () => {
      out = await result.current.exportAttendance(REQUEST, "odoo.csv");
    });
    expect(out).toBeNull();
    expect(result.current.error).toBeNull();
  });

  it("surfaces a backend error", async () => {
    listAttendanceDays.mockResolvedValue(DAYS);
    exportAttendance.mockRejectedValue(
      new AppError("VALIDATION", "employeeName is required"),
    );
    const { result } = renderHook(() => useAttendanceExport("p1", true), {
      wrapper,
    });
    await act(async () => {
      await result.current.exportAttendance(REQUEST, "odoo.csv");
    });
    expect(result.current.error).not.toBeNull();
  });
});
