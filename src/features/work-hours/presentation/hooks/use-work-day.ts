import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { isAppError } from "@/core/errors";
import { TauriWorkEntryRepository } from "@/features/work-hours/data/repositories/tauri-work-entry.repository";
import {
  toWorkEntryInput,
  type WorkDaySchedule,
  type WorkEntry,
  type WorkEntryInput,
} from "@/features/work-hours/domain/entities/work-hours";
import { makeGetWorkEntriesUseCase } from "@/features/work-hours/domain/use-cases/get-work-entries";
import { makeGetWorkScheduleUseCase } from "@/features/work-hours/domain/use-cases/get-work-schedule";
import { makeSetWorkEntriesUseCase } from "@/features/work-hours/domain/use-cases/set-work-entries";
import { makeSetWorkScheduleUseCase } from "@/features/work-hours/domain/use-cases/set-work-schedule";

const entryRepo = new TauriWorkEntryRepository();
const getEntriesUseCase = makeGetWorkEntriesUseCase(entryRepo);
const setEntriesUseCase = makeSetWorkEntriesUseCase(entryRepo);
const getScheduleUseCase = makeGetWorkScheduleUseCase(entryRepo);
const setScheduleUseCase = makeSetWorkScheduleUseCase(entryRepo);

/** Delay between the last local mutation and the persisting replace-all call. */
const SAVE_DEBOUNCE_MS = 600;

export interface UseWorkDayValue {
  entries: WorkEntry[];
  totalMinutes: number;
  /** The day's start/end times, `null` until the user sets them. */
  schedule: WorkDaySchedule | null;
  isLoading: boolean;
  /** `true` while a save is pending or in flight (subtle indicator). */
  isSaving: boolean;
  error: string | null;
  addEntry: (input: WorkEntryInput) => void;
  updateMinutes: (id: string, minutes: number) => void;
  removeEntry: (id: string) => void;
  /** Swap the entry with its neighbour (direction -1 = up, +1 = down). */
  moveEntry: (id: string, direction: -1 | 1) => void;
  /** Persist the day's start time (saved immediately); the backend stores
   * the recomputed end alongside it. */
  setSchedule: (startMinutes: number) => void;
}

/**
 * Local state of a day's work entries with debounced live persistence: every
 * mutation applies instantly to the UI, then the full entry set is replaced
 * backend-side once edits settle. Pending edits are flushed on unmount so
 * navigating back never loses the last change.
 */
export function useWorkDay(
  presenceId: string,
  onSessionExpired?: () => void,
): UseWorkDayValue {
  const [entries, setEntries] = useState<WorkEntry[]>([]);
  const [schedule, setScheduleState] = useState<WorkDaySchedule | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Refs so the debounce timer and the unmount flush always see latest state.
  const entriesRef = useRef<WorkEntry[]>([]);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const dirtyRef = useRef(false);
  // Bumped on every local mutation; a save response is only applied when no
  // newer edit happened while the call was in flight.
  const versionRef = useRef(0);

  const onSessionExpiredRef = useRef(onSessionExpired);
  onSessionExpiredRef.current = onSessionExpired;

  const handleError = useCallback((e: unknown) => {
    if (isAppError(e) && e.code === "SESSION_EXPIRED") {
      onSessionExpiredRef.current?.();
      return;
    }
    setError(isAppError(e) ? e.message : "Une erreur est survenue");
  }, []);

  const persist = useCallback(async () => {
    if (!dirtyRef.current) return;
    dirtyRef.current = false;
    const version = versionRef.current;
    try {
      const saved = await setEntriesUseCase(
        presenceId,
        entriesRef.current.map(toWorkEntryInput),
      );
      // Adopt the canonical rows (fresh ids) and the recomputed schedule only
      // if nothing changed meanwhile.
      if (versionRef.current === version) {
        entriesRef.current = saved.entries;
        setEntries(saved.entries);
        setScheduleState(saved.schedule);
        setError(null);
      }
    } catch (e) {
      dirtyRef.current = true;
      handleError(e);
    } finally {
      if (versionRef.current === version) setIsSaving(false);
    }
  }, [presenceId, handleError]);

  const scheduleSave = useCallback(() => {
    dirtyRef.current = true;
    setIsSaving(true);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      timerRef.current = null;
      void persist();
    }, SAVE_DEBOUNCE_MS);
  }, [persist]);

  const mutate = useCallback(
    (next: (prev: WorkEntry[]) => WorkEntry[]) => {
      versionRef.current += 1;
      entriesRef.current = next(entriesRef.current);
      setEntries(entriesRef.current);
      scheduleSave();
    },
    [scheduleSave],
  );

  // Load the day's entries + schedule; flush any pending save on leave.
  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    Promise.all([getEntriesUseCase(presenceId), getScheduleUseCase(presenceId)])
      .then(([list, sched]) => {
        if (cancelled) return;
        entriesRef.current = list;
        setEntries(list);
        setScheduleState(sched);
      })
      .catch((e) => {
        if (!cancelled) handleError(e);
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
      void persist();
    };
  }, [presenceId, persist, handleError]);

  const addEntry = useCallback(
    (input: WorkEntryInput) => {
      mutate((prev) => [
        ...prev,
        {
          id: crypto.randomUUID(),
          title: input.title,
          description: input.description ?? null,
          minutes: input.minutes,
          color: input.color,
          position: prev.length,
        },
      ]);
    },
    [mutate],
  );

  const updateMinutes = useCallback(
    (id: string, minutes: number) => {
      mutate((prev) => prev.map((e) => (e.id === id ? { ...e, minutes } : e)));
    },
    [mutate],
  );

  const removeEntry = useCallback(
    (id: string) => {
      mutate((prev) =>
        prev.filter((e) => e.id !== id).map((e, i) => ({ ...e, position: i })),
      );
    },
    [mutate],
  );

  const moveEntry = useCallback(
    (id: string, direction: -1 | 1) => {
      mutate((prev) => {
        const from = prev.findIndex((e) => e.id === id);
        const to = from + direction;
        if (from < 0 || to < 0 || to >= prev.length) return prev;
        const next = [...prev];
        [next[from], next[to]] = [next[to], next[from]];
        return next.map((e, i) => ({ ...e, position: i }));
      });
    },
    [mutate],
  );

  // Saved immediately (a time picker commit is a single, settled action).
  // The displayed schedule is always the saved row returned by the backend.
  const setSchedule = useCallback(
    (startMinutes: number) => {
      setScheduleUseCase(presenceId, startMinutes)
        .then((saved) => {
          setScheduleState(saved);
          setError(null);
        })
        .catch(handleError);
    },
    [presenceId, handleError],
  );

  const totalMinutes = useMemo(
    () => entries.reduce((sum, e) => sum + e.minutes, 0),
    [entries],
  );

  return {
    entries,
    totalMinutes,
    schedule,
    isLoading,
    isSaving,
    error,
    addEntry,
    updateMinutes,
    removeEntry,
    moveEntry,
    setSchedule,
  };
}
