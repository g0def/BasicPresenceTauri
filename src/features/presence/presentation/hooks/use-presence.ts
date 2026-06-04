import { useContext } from "react";

import {
  PresenceContext,
  type PresenceContextValue,
} from "@/features/presence/presentation/context/presence-context";

export function usePresence(): PresenceContextValue {
  const ctx = useContext(PresenceContext);
  if (!ctx) {
    throw new Error("usePresence must be used within <PresenceProvider>");
  }
  return ctx;
}
