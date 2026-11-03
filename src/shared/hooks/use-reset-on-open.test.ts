import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useResetOnOpen } from "@/shared/hooks/use-reset-on-open";

describe("useResetOnOpen", () => {
  it("resets when the dialog opens, not while it stays open or closed", () => {
    const reset = vi.fn();
    const { rerender } = renderHook(
      ({ open }: { open: boolean }) => useResetOnOpen(open, null, reset),
      { initialProps: { open: false } },
    );
    expect(reset).not.toHaveBeenCalled();

    rerender({ open: true });
    expect(reset).toHaveBeenCalledTimes(1);

    rerender({ open: true });
    rerender({ open: false });
    expect(reset).toHaveBeenCalledTimes(1);

    rerender({ open: true });
    expect(reset).toHaveBeenCalledTimes(2);
  });

  it("resets when an open dialog starts targeting another item", () => {
    const reset = vi.fn();
    const { rerender } = renderHook(
      ({ target }: { target: string }) => useResetOnOpen(true, target, reset),
      { initialProps: { target: "a" } },
    );
    expect(reset).toHaveBeenCalledTimes(1);

    rerender({ target: "a" });
    expect(reset).toHaveBeenCalledTimes(1);

    rerender({ target: "b" });
    expect(reset).toHaveBeenCalledTimes(2);
  });
});
