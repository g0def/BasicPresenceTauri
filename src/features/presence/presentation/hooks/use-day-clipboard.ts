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
  /** Whether overwriting already-filled days has been approved for this
   * session; set by `approveOverwrite`, cleared when the session ends. */
  overwriteApproved: boolean;
  /** Enter "pick a source day" mode (toolbar toggle turns orange). */
  arm: () => void;
  /** Return to normal mode and drop the copied day. */
  disarm: () => void;
  /** Approve overwriting filled days for the rest of the session (granted once
   * the user confirms the first replace). */
  approveOverwrite: () => void;
  /** Capture a day's full encoding and switch to paste mode (toggle red). */
  copyDay: (presence: Presence) => Promise<void>;
  /** Replay the copied encoding onto a target day (UTC-midnight key),
   * overwriting any existing presence. The caller gates the overwrite of
   * already-filled days behind a confirmation (see `overwriteApproved`). */
  pasteOnto: (dayKey: number) => Promise<void>;
}

/**
 * Session-scoped "copy a day" clipboard, driven from a single toolbar toggle:
 * arm it, click a fully-encoded day to copy it, then click empty days to
 * replay its presence type, commute trips, start time and tasks. Lets the user
 * fill a sparse calendar quickly without re-encoding identical days one by one.
 */
export function useDayClipboard(): UseDayClipboardValue {
  const { presencesByDay, setPresence, getPresenceTrips, reload } =
    usePresence();
  const [mode, setMode] = useState<ClipboardMode>("idle");
  const [copied, setCopied] = useState<CopiedDay | null>(null);
  const [overwriteApproved, setOverwriteApproved] = useState(false);

  const arm = useCallback(() => setMode("picking"), []);

  const disarm = useCallback(() => {
    setMode("idle");
    setCopied(null);
    setOverwriteApproved(false);
  }, []);

  const approveOverwrite = useCallback(() => setOverwriteApproved(true), []);

  const copyDay = useCallback(
    async (presence: Presence): Promise<void> => {
      // A fresh source day starts a new paste session: re-require confirmation
      // before overwriting already-filled days.
      setOverwriteApproved(false);
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
      // Remember the day's prior type before the upsert overwrites it, so we
      // know whether we need to wipe leftover work hours below.
      const prior = presencesByDay.get(dayKey);
      // setPresence reports its own errors and returns null on failure; the new
      // presence id is needed to attach the schedule and tasks.
      const saved = await setPresence(dayKey, copied.type, copied.trips);
      if (!saved) return;
      // Full replace: the pasted day must end up an exact copy of the source.
      // Work hours only exist on office/remote days, so (re)write them when the
      // copied day is a work day, and wipe leftovers when a non-work day
      // overwrites a former work day. set_work_entries is replace-all, so an
      // empty list clears any residual tasks; skip the IPC entirely otherwise.
      const priorWasWork = prior ? isWorkType(prior.type) : false;
      if (isWorkType(copied.type) || priorWasWork) {
        try {
          // Set the start first, then the entries: set_work_entries recomputes
          // the end as start + sum(minutes).
          if (isWorkType(copied.type) && copied.startMinutes !== null) {
            await setWorkSchedule(saved.id, copied.startMinutes);
          }
          await setWorkEntries(
            saved.id,
            isWorkType(copied.type) ? copied.entries : [],
          );
        } catch {
          // The presence is set even if hours fail to attach; leave it as is.
        }
      }
      // Re-pull so the calendar reflects the recomputed work minutes / CO2.
      await reload();
    },
    [copied, presencesByDay, setPresence, reload],
  );

  return {
    mode,
    copied,
    overwriteApproved,
    arm,
    disarm,
    approveOverwrite,
    copyDay,
    pasteOnto,
  };
}
