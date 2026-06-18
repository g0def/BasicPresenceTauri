import { useCallback, useState } from "react";

import type {
  Presence,
  PresenceTrip,
  PresenceType,
} from "@/features/presence/domain/entities/presence";
import { usePresence } from "@/features/presence/presentation/hooks/use-presence";
import {
  toWorkEntryInput,
  type WorkEntryInput,
} from "@/features/work-hours/domain/entities/work-hours";
import {
  getWorkEntries,
  getWorkSchedule,
  setWorkEntries,
  setWorkSchedule,
} from "@/features/work-hours/presentation/work-day-actions";

/** Office/remote days carry a commute, a schedule and work entries; the other
 * types are just a presence stamp. */
const isWorkType = (type: PresenceType): boolean =>
  type === "office" || type === "remote";

/**
 * - `idle`: normal calendar (clicking a day opens its dialog).
 * - `picking`: armed to choose the source day (toolbar toggle is orange);
 *   clicking an encoded day copies it.
 * - `pasting`: a day is copied (toggle is red with a cross); clicking empty
 *   days replays it onto them.
 */
export type ClipboardMode = "idle" | "picking" | "pasting";

/** A full day encoding captured for pasting onto other days. */
export interface CopiedDay {
  type: PresenceType;
  trips: PresenceTrip[];
  /** Work-day start, in minutes since midnight; `null` for non-work days. */
  startMinutes: number | null;
  /** Work entries (tasks) to replay; empty for non-work days. */
  entries: WorkEntryInput[];
}

export interface UseDayClipboardValue {
  mode: ClipboardMode;
  /** The currently copied day, set once a source day has been picked. */
  copied: CopiedDay | null;
  /** Enter "pick a source day" mode (toolbar toggle turns orange). */
  arm: () => void;
  /** Return to normal mode and drop the copied day. */
  disarm: () => void;
  /** Capture a day's full encoding and switch to paste mode (toggle red). */
  copyDay: (presence: Presence) => Promise<void>;
  /** Replay the copied encoding onto a target day (UTC-midnight key). The
   * caller is responsible for skipping days that already have a presence. */
  pasteOnto: (dayKey: number) => Promise<void>;
}

/**
 * Session-scoped "copy a day" clipboard, driven from a single toolbar toggle:
 * arm it, click a fully-encoded day to copy it, then click empty days to
 * replay its presence type, commute trips, start time and tasks. Lets the user
 * fill a sparse calendar quickly without re-encoding identical days one by one.
 */
export function useDayClipboard(): UseDayClipboardValue {
  const { setPresence, getPresenceTrips, reload } = usePresence();
  const [mode, setMode] = useState<ClipboardMode>("idle");
  const [copied, setCopied] = useState<CopiedDay | null>(null);

  const arm = useCallback(() => setMode("picking"), []);

  const disarm = useCallback(() => {
    setMode("idle");
    setCopied(null);
  }, []);

  const copyDay = useCallback(
    async (presence: Presence): Promise<void> => {
      try {
        const trips = await getPresenceTrips(presence.id);
        let startMinutes: number | null = null;
        let entries: WorkEntryInput[] = [];
        if (isWorkType(presence.type)) {
          const [schedule, list] = await Promise.all([
            getWorkSchedule(presence.id),
            getWorkEntries(presence.id),
          ]);
          startMinutes = schedule?.startMinutes ?? null;
          entries = list.map(toWorkEntryInput);
        }
        setCopied({ type: presence.type, trips, startMinutes, entries });
        setMode("pasting");
      } catch {
        // Best-effort: stay armed so another source day can be tried.
      }
    },
    [getPresenceTrips],
  );

  const pasteOnto = useCallback(
    async (dayKey: number): Promise<void> => {
      if (!copied) return;
      // setPresence reports its own errors and returns null on failure; the new
      // presence id is needed to attach the schedule and tasks.
      const saved = await setPresence(dayKey, copied.type, copied.trips);
      if (!saved) return;
      if (isWorkType(copied.type)) {
        try {
          // Set the start first, then the entries: set_work_entries recomputes
          // the end as start + sum(minutes).
          if (copied.startMinutes !== null) {
            await setWorkSchedule(saved.id, copied.startMinutes);
          }
          if (copied.entries.length > 0) {
            await setWorkEntries(saved.id, copied.entries);
          }
        } catch {
          // The presence is set even if hours fail to attach; leave it as is.
        }
      }
      // Re-pull so the calendar reflects the recomputed work minutes / CO2.
      await reload();
    },
    [copied, setPresence, reload],
  );

  return { mode, copied, arm, disarm, copyDay, pasteOnto };
}
