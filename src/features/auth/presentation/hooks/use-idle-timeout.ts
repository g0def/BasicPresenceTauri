import { useEffect, useRef } from "react";

/**
 * Logs the user out after `idleMs` without any interaction. Listens for the
 * usual activity signals and re-arms a timer on each one; firing the timer
 * calls `onIdle`. Pass `enabled = false` (e.g. when unauthenticated) to make
 * the hook inert and tear the listeners down.
 *
 * Complements the absolute session TTL: the backend still caps the session at
 * its hard limit, this only shortens it when the app is left unattended.
 */
const ACTIVITY_EVENTS = [
  "mousemove",
  "mousedown",
  "keydown",
  "touchstart",
  "scroll",
  "wheel",
] as const;

export function useIdleTimeout(
  idleMs: number,
  onIdle: () => void,
  enabled: boolean,
): void {
  // Keep the latest callback without re-arming listeners on every render.
  const onIdleRef = useRef(onIdle);
  onIdleRef.current = onIdle;

  useEffect(() => {
    if (!enabled) return;

    let timer = window.setTimeout(() => onIdleRef.current(), idleMs);

    const reset = () => {
      window.clearTimeout(timer);
      timer = window.setTimeout(() => onIdleRef.current(), idleMs);
    };

    // A backgrounded window stops firing activity events; treat coming back
    // to the foreground as activity so we don't log out a tab the user just
    // switched away from and back to.
    const onVisibility = () => {
      if (document.visibilityState === "visible") reset();
    };

    for (const event of ACTIVITY_EVENTS) {
      window.addEventListener(event, reset, { passive: true });
    }
    document.addEventListener("visibilitychange", onVisibility);

    return () => {
      window.clearTimeout(timer);
      for (const event of ACTIVITY_EVENTS) {
        window.removeEventListener(event, reset);
      }
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, [idleMs, enabled]);
}
