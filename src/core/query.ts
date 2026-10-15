import { useCallback } from "react";
import { useMutation } from "@tanstack/react-query";

import { isAppError } from "@/core/errors";

/**
 * Whether an error means the backend locked the vault (session over). Those
 * are handled once, globally, by the query client (logout) — features must not
 * surface them as a regular error message.
 */
export function isSessionExpired(e: unknown): boolean {
  return isAppError(e) && e.code === "SESSION_EXPIRED";
}

/** User-facing message of an error (backend messages are already safe). */
export function errorMessage(e: unknown, fallback: string): string {
  return isAppError(e) ? e.message : fallback;
}

/**
 * Run a backend write through TanStack Query's mutation machinery, so the
 * global error handling (SESSION_EXPIRED → logout) applies to it, and expose
 * its in-flight state. One runner per provider: `isPending` reads as
 * "a write is in flight".
 */
export function useCommand() {
  const mutation = useMutation<unknown, unknown, () => Promise<unknown>>({
    mutationFn: (work) => work(),
  });
  const { mutateAsync } = mutation;
  const run = useCallback(
    <T>(work: () => Promise<T>): Promise<T> => mutateAsync(work) as Promise<T>,
    [mutateAsync],
  );
  return { run, isPending: mutation.isPending };
}
