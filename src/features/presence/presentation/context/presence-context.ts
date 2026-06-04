import { createContext } from "react";

import type {
  Presence,
  PresenceType,
} from "@/features/presence/domain/entities/presence";

export interface PresenceContextValue {
  /** Presences of the active profile, keyed by `day` (UTC-midnight epoch ms). */
  presencesByDay: Map<number, Presence>;
  /** True while the presence list is (re)loading. */
  isLoading: boolean;
  /** True while a set/delete is in flight. */
  isSubmitting: boolean;
  error: string | null;
  /** Create or update the presence type for a day. Returns `true` on success. */
  setPresence: (day: number, type: PresenceType) => Promise<boolean>;
  /** Delete a presence by id. Returns `true` on success. */
  deletePresence: (id: string) => Promise<boolean>;
  clearError: () => void;
}

export const PresenceContext = createContext<PresenceContextValue | null>(null);
