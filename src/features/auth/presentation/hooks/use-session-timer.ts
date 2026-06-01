import { useEffect, useState } from "react";

import { SESSION_TICK_MS } from "@/core/config";

/**
 * Returns the milliseconds remaining until `expiresAt`, ticking every second.
 * Calls `onExpire` once the deadline passes. Pass `expiresAt = null` when
 * unauthenticated (the timer is inert).
 *
 * Driven by the absolute `expiresAt` (not a decrementing counter) so OS sleep
 * or clock changes can never grant extra session time.
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
