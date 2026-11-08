import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { AppError } from "@/core/errors";
import { useFeatureError } from "@/core/query";

const failure = (message: string) => new AppError("INTERNAL", message);

describe("useFeatureError", () => {
  it("derives the message of a failed query, and drops it once it recovers", () => {
    const { result, rerender } = renderHook<
      ReturnType<typeof useFeatureError>,
      { err: unknown }
    >(({ err }) => useFeatureError("generic", err), {
      initialProps: { err: failure("load failed") },
    });
    expect(result.current.error).toBe("load failed");

    rerender({ err: null });
    expect(result.current.error).toBeNull();
  });

  it("hides a dismissed query error until the query fails again", () => {
    const first = failure("first");
    const { result, rerender } = renderHook<
      ReturnType<typeof useFeatureError>,
      { err: unknown }
    >(({ err }) => useFeatureError("generic", err), {
      initialProps: { err: first },
    });

    act(() => result.current.clearError());
    expect(result.current.error).toBeNull();

    rerender({ err: failure("second") });
    expect(result.current.error).toBe("second");
  });

  it("shows an action error first, and clears it with the query errors", () => {
    // Same object across renders, as TanStack Query's `query.error` is.
    const loadError = failure("load");
    const { result } = renderHook(() => useFeatureError("generic", loadError));

    act(() => result.current.reportError(failure("save failed")));
    expect(result.current.error).toBe("save failed");

    act(() => result.current.clearError());
    expect(result.current.error).toBeNull();
  });

  it("never surfaces a session expiry (handled globally by logout)", () => {
    const expired = new AppError("SESSION_EXPIRED", "locked");
    const { result } = renderHook(() => useFeatureError("generic", expired));
    expect(result.current.error).toBeNull();

    act(() => result.current.reportError(expired));
    expect(result.current.error).toBeNull();
  });

  it("falls back to the generic message for unknown errors", () => {
    const { result } = renderHook(() =>
      useFeatureError("generic", null, new Error("boom")),
    );
    expect(result.current.error).toBe("generic");
  });
});
