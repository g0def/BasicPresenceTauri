import { useContext } from "react";

import {
  CommuteContext,
  type CommuteContextValue,
} from "@/features/commute/presentation/context/commute-context";

export function useCommute(): CommuteContextValue {
  const ctx = useContext(CommuteContext);
  if (!ctx) {
    throw new Error("useCommute must be used within <CommuteProvider>");
  }
  return ctx;
}
