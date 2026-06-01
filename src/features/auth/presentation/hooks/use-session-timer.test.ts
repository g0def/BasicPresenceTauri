import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useSessionTimer } from "@/features/auth/presentation/hooks/use-session-timer";

describe("useSessionTimer", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(0);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("counts down from the absolute expiry", () => {
    const onExpire = vi.fn();
    const { result } = renderHook(() => useSessionTimer(5_000, onExpire));

    expect(result.current).toBe(5_000);

    act(() => {
      vi.advanceTimersByTime(3_000);
    });
    expect(result.current).toBe(2_000);
    expect(onExpire).not.toHaveBeenCalled();
  });

  it("fires onExpire once the deadline passes", () => {
    const onExpire = vi.fn();
    const { result } = renderHook(() => useSessionTimer(2_000, onExpire));

    act(() => {
      vi.advanceTimersByTime(3_000);
    });

    expect(onExpire).toHaveBeenCalled();
    expect(result.current).toBe(0);
  });
});
