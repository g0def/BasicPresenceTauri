import { useMemo, type ReactNode } from "react";

import { useProfileSettings } from "@/features/profile-settings/presentation/hooks/use-profile-settings";
import {
  CellDisplayContext,
  type CellDisplayMode,
} from "./cell-display-context";

/**
 * Shares the calendar cell display preference (CO2 vs work hours) across the
 * tree so the header menu and the calendar — which live in different subtrees —
 * stay in sync. The value is now sourced from (and written back to) the active
 * profile's settings, so the choice is per-profile and persisted in the vault.
 */
export function CellDisplayProvider({ children }: { children: ReactNode }) {
  const { settings, update } = useProfileSettings();

  const value = useMemo(
    () => ({
      mode: settings.cellDisplayMode,
      setMode: (next: CellDisplayMode) =>
        void update({ cellDisplayMode: next }),
    }),
    [settings.cellDisplayMode, update],
  );

  return (
    <CellDisplayContext.Provider value={value}>
      {children}
    </CellDisplayContext.Provider>
  );
}
