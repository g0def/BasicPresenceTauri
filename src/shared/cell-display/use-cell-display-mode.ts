import { useContext } from "react";

import { CellDisplayContext } from "./cell-display-context";

export type { CellDisplayMode } from "./cell-display-context";

export function useCellDisplayMode() {
  const ctx = useContext(CellDisplayContext);
  if (!ctx) {
    throw new Error(
      "useCellDisplayMode must be used within CellDisplayProvider",
    );
  }
  return ctx;
}
