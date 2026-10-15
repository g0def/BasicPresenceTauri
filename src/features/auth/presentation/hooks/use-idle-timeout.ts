import { useEffect, useRef } from "react";

/**
 * Logs the user out after `idleMs` without any interaction, and reports real
 * activity (throttled to one call per `activityPingMs`) through `onActivity` —
 * used to keep the backend's own idle timer in step with the UI.
 *
 * Pass `enabled = false` (e.g. when unauthenticated) to make the hook inert and
 * tear the listeners down.
 *
 * Complements the absolute session TTL: the backend still caps the session at
 * its hard limit and enforces its own inactivity limit; this only shortens the
 * session when the app is left unattended.
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
  onActivity?: () => void,
  activityPingMs = 60_000,
): void {
  // Keep the latest callbacks without re-arming listeners on every render.
  const onIdleRef = useRef(onIdle);
  onIdleRef.current = onIdle;
  const onActivityRef = useRef(onActivity);
  onActivityRef.current = onActivity;

  useEffect(() => {
    if (!enabled) return;

    let lastActivity = Date.now();
    let lastPing = lastActivity;
    let timer = window.setTimeout(() => onIdleRef.current(), idleMs);

    const arm = (delay: number) => {
      window.clearTimeout(timer);
      timer = window.setTimeout(() => onIdleRef.current(), delay);
    };

    const onUserActivity = () => {
      const now = Date.now();
      lastActivity = now;
      arm(idleMs);
      if (now - lastPing >= activityPingMs) {
        lastPing = now;
        onActivityRef.current?.();
      }
    };

    // A backgrounded window stops firing activity events and its timers may be
    // throttled. Coming back is NOT activity: judge by the time actually
    // elapsed since the last interaction.
    const onVisibility = () => {
      if (document.visibilityState !== "visible") return;
      const idleFor = Date.now() - lastActivity;
      if (idleFor >= idleMs) {
        window.clearTimeout(timer);
        onIdleRef.current();
      } else {
        arm(idleMs - idleFor);
      }
    };

    for (const event of ACTIVITY_EVENTS) {
      window.addEventListener(event, onUserActivity, { passive: true });
    }
    document.addEventListener("visibilitychange", onVisibility);

    return () => {
      window.clearTimeout(timer);
      for (const event of ACTIVITY_EVENTS) {
        window.removeEventListener(event, onUserActivity);
      }
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, [idleMs, enabled, activityPingMs]);
}
