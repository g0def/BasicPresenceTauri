import { useState } from "react";

/**
 * Resets a dialog's form state when it opens — or, while open, when it starts
 * targeting another item — in the same render instead of in an effect (which
 * would paint the stale state first, then render again). This is React's
 * "adjusting some state when a prop changes" pattern:
 * https://react.dev/learn/you-might-not-need-an-effect#adjusting-some-state-when-a-prop-changes
 *
 * `reset` runs during render: it may only call state setters of the calling
 * component (or of hooks it owns), never another component's (e.g. a context
 * provider's).
 */
export function useResetOnOpen(
  open: boolean,
  target: unknown,
  reset: () => void,
): void {
  const [seen, setSeen] = useState<{ open: boolean; target: unknown }>({
    open: false,
    target,
  });
  if (seen.open !== open || (open && !Object.is(seen.target, target))) {
    setSeen({ open, target });
    if (open) reset();
  }
}
