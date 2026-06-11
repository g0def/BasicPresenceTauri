import { useCallback, useState } from "react";

const STORAGE_KEY = "auto-update";

function readStoredPreference(): boolean {
  if (typeof localStorage === "undefined") return true;
  const stored = localStorage.getItem(STORAGE_KEY);
  // Default to enabled (security patches ship by default; user can opt out).
  return stored === null ? true : stored === "true";
}

export function useAutoUpdatePreference() {
  const [enabled, setEnabledState] = useState<boolean>(readStoredPreference);

  const setEnabled = useCallback((next: boolean) => {
    localStorage.setItem(STORAGE_KEY, String(next));
    setEnabledState(next);
  }, []);

  const toggleEnabled = useCallback(() => {
    setEnabledState((prev) => {
      const next = !prev;
      localStorage.setItem(STORAGE_KEY, String(next));
      return next;
    });
  }, []);

  return { enabled, setEnabled, toggleEnabled };
}
