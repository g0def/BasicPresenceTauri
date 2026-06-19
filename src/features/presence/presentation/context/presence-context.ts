import { createContext } from "react";

import type {
  Presence,
  PresenceTrip,
  PresenceType,
} from "@/features/presence/domain/entities/presence";

export interface PresenceContextValue {
  /** Presences of the active profile, keyed by `day` (UTC-midnight epoch ms). */
  presencesByDay: Map<number, Presence>;
  /** The currently selected month in the calendar. */
  currentMonth: Date;
  /** Set the currently selected month. */
  setCurrentMonth: (date: Date) => void;
  /** Force-refresh the presence list from the database. */
  reload: () => Promise<void>;
  /** True while the presence list is (re)loading. */
  isLoading: boolean;
  /** True while a set/delete is in flight. */
  isSubmitting: boolean;
  error: string | null;
  /** Create or update the presence for a day, optionally with commute trips
   * (office/remote). Returns the saved presence on success, `null` on failure
   * (the caller needs the new id to attach work hours when pasting a day). */
  setPresence: (
    day: number,
    type: PresenceType,
    trips?: PresenceTrip[],
  ) => Promise<Presence | null>;
  /** Delete a presence by id. Returns `true` on success. */
  deletePresence: (id: string) => Promise<boolean>;
  /** Fetch the commute trips recorded for a presence day (for re-editing). */
  getPresenceTrips: (presenceId: string) => Promise<PresenceTrip[]>;
  clearError: () => void;
}

export const PresenceContext = createContext<PresenceContextValue | null>(null);
