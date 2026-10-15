import { useMemo } from "react";

import { useContainer } from "@/core/di";

/**
 * A day's work entries and schedule, read/written through the injected
 * work-entry port — by the work-hours editor and by the day-copy clipboard of
 * the presence calendar. Lives in the shared `entities` layer so neither
 * feature depends on the other.
 */
export function useWorkDayActions() {
  const { workEntryRepository: repo } = useContainer();
  return useMemo(
    () => ({
      getWorkEntries: (presenceId: string) => repo.listByPresence(presenceId),
      setWorkEntries: repo.replaceForPresence.bind(repo),
      getWorkSchedule: (presenceId: string) => repo.getSchedule(presenceId),
      setWorkSchedule: repo.setSchedule.bind(repo),
    }),
    [repo],
  );
}
