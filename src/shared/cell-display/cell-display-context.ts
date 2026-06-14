import { createContext } from "react";

/** What each calendar day cell renders: the CO2 footprint or the work hours. */
export type CellDisplayMode = "co2" | "hours";

export interface CellDisplayContextValue {
  mode: CellDisplayMode;
  setMode: (next: CellDisplayMode) => void;
}

export const CellDisplayContext = createContext<CellDisplayContextValue | null>(
  null,
);
