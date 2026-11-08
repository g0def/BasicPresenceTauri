import { useContext } from "react";

import {
  PresenceContext,
  type PresenceContextValue,
} from "@/entities/presence/model/presence-context";

export function usePresence(): PresenceContextValue {
  const ctx = useContext(PresenceContext);
  if (!ctx) {
    throw new Error("usePresence must be used within <PresenceProvider>");
  }
  return ctx;
}
