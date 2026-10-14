import { useEffect, useState } from "react";

import { SESSION_TICK_MS } from "@/core/config";

/**
 * Returns the milliseconds remaining until `expiresAt`, ticking every second.
 * Calls `onExpire` once the deadline passes. Pass `expiresAt = null` when
 * unauthenticated (the timer is inert).
 *
 * Driven by the absolute `expiresAt` rather than a decrementing counter, so
 * waking from sleep past the deadline logs out immediately instead of resuming
 * where it left off.
 *
 * This countdown is **display and convenience only**. It reads `Date.now()`,
 * the wall clock, which the user can move — so it is not, on its own, a control.
 * The deadline that counts is enforced in Rust: every vault-touching command
 * goes through `require_session`, against a clock that cannot be wound back
 * (see `infrastructure/clock.rs`). If the two ever disagree, the backend wins.
 */
export function useSessionTimer(
  expiresAt: number | null,
  onExpire: () => void,
): number {
  const [remainingMs, setRemainingMs] = useState(() =>
    expiresAt ? Math.max(0, expiresAt - Date.now()) : 0,
  );

  useEffect(() => {
    if (expiresAt == null) {
      setRemainingMs(0);
      return;
    }

    const tick = () => {
      const left = expiresAt - Date.now();
      if (left <= 0) {
        setRemainingMs(0);
        onExpire();
      } else {
        setRemainingMs(left);
      }
    };

    tick(); // immediate check (covers wake-from-sleep / remount past deadline)
    const id = window.setInterval(tick, SESSION_TICK_MS);
    return () => window.clearInterval(id);
  }, [expiresAt, onExpire]);

  return remainingMs;
}
