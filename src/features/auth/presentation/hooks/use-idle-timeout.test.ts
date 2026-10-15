import { renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useIdleTimeout } from "@/features/auth/presentation/hooks/use-idle-timeout";

function setVisibility(state: DocumentVisibilityState) {
  Object.defineProperty(document, "visibilityState", {
    configurable: true,
    get: () => state,
  });
  document.dispatchEvent(new Event("visibilitychange"));
}

describe("useIdleTimeout", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
    setVisibility("visible");
  });

  it("fires after the idle window and is re-armed by activity", () => {
    const onIdle = vi.fn();
    renderHook(() => useIdleTimeout(1_000, onIdle, true));

    vi.advanceTimersByTime(900);
    window.dispatchEvent(new Event("mousemove"));
    vi.advanceTimersByTime(900);
    expect(onIdle).not.toHaveBeenCalled();

    vi.advanceTimersByTime(100);
    expect(onIdle).toHaveBeenCalledOnce();
  });

  it("returning to the foreground after the idle window logs out at once", () => {
    const onIdle = vi.fn();
    renderHook(() => useIdleTimeout(1_000, onIdle, true));

    setVisibility("hidden");
    // Background timers may be throttled: only the wall clock moves.
    vi.setSystemTime(Date.now() + 5_000);
    setVisibility("visible");

    expect(onIdle).toHaveBeenCalledOnce();
  });

  it("reports activity to the backend at most once per ping window", () => {
    const onActivity = vi.fn();
    renderHook(() => useIdleTimeout(10_000, vi.fn(), true, onActivity, 1_000));

    window.dispatchEvent(new Event("keydown"));
    expect(onActivity).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1_000);
    window.dispatchEvent(new Event("keydown"));
    window.dispatchEvent(new Event("keydown"));
    expect(onActivity).toHaveBeenCalledOnce();
  });

  it("is inert while disabled", () => {
    const onIdle = vi.fn();
    renderHook(() => useIdleTimeout(1_000, onIdle, false));
    vi.advanceTimersByTime(5_000);
    expect(onIdle).not.toHaveBeenCalled();
  });
});
