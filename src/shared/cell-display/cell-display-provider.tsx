import { useCallback, useState, type ReactNode } from "react";

import {
  CellDisplayContext,
  type CellDisplayMode,
} from "./cell-display-context";

const STORAGE_KEY = "cell-display-mode";

function readStoredMode(): CellDisplayMode {
  if (typeof localStorage === "undefined") return "co2";
  return localStorage.getItem(STORAGE_KEY) === "hours" ? "hours" : "co2";
}

/**
 * Shares the calendar cell display preference (CO2 vs work hours) across the
 * tree so the header menu and the calendar — which live in different subtrees —
 * stay in sync. Choice persisted in localStorage; defaults to "co2" to preserve
 * the original calendar behavior.
 */
export function CellDisplayProvider({ children }: { children: ReactNode }) {
  const [mode, setModeState] = useState<CellDisplayMode>(readStoredMode);

  const setMode = useCallback((next: CellDisplayMode) => {
    localStorage.setItem(STORAGE_KEY, next);
    setModeState(next);
  }, []);

  return (
    <CellDisplayContext.Provider value={{ mode, setMode }}>
      {children}
    </CellDisplayContext.Provider>
  );
}
