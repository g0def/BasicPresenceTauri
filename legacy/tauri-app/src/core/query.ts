import { useCallback, useState } from "react";
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
 * Error state of a feature provider: the last failed action, else the first
 * failed query. Query errors are derived during render (never copied into
 * state by an effect), so a query that recovers stops showing its error.
 * `clearError` dismisses both — the dismissed query errors stay hidden until
 * the query fails again (a new error object).
 */
export function useFeatureError(
  fallback: string,
  queryError: unknown,
  otherQueryError: unknown = null,
) {
  const [actionError, setActionError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState<readonly unknown[]>([]);

  const shownQueryError =
    [queryError, otherQueryError].find(
      (e) => e != null && !isSessionExpired(e) && !dismissed.includes(e),
    ) ?? null;

  const reportError = useCallback(
    (e: unknown) => {
      if (!isSessionExpired(e)) setActionError(errorMessage(e, fallback));
    },
    [fallback],
  );

  const clearError = useCallback(() => {
    setActionError(null);
    setDismissed([queryError, otherQueryError]);
  }, [queryError, otherQueryError]);

  const error =
    actionError ??
    (shownQueryError ? errorMessage(shownQueryError, fallback) : null);
  return { error, reportError, clearError };
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
