import { createContext } from "react";

import type { CellDisplayMode } from "./cell-display-mode";

export type { CellDisplayMode };

export interface CellDisplayContextValue {
  mode: CellDisplayMode;
  setMode: (next: CellDisplayMode) => void;
}

export const CellDisplayContext = createContext<CellDisplayContextValue | null>(
  null,
);
