import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query";

import { isSessionExpired } from "@/core/query";

/** Set by the app once auth is mounted (see App.tsx). */
let onSessionExpired: (() => void) | null = null;

export function setSessionExpiredHandler(handler: (() => void) | null): void {
  onSessionExpired = handler;
}

function handleError(error: unknown): void {
  // A locked vault means the session is over, wherever it surfaced: log out
  // once, centrally (the router guard then redirects to /login).
  if (isSessionExpired(error)) onSessionExpired?.();
}

/**
 * The app's single query client. Data lives in the encrypted vault and the UI
 * is its only writer, so cached data never goes stale behind our back:
 * queries are invalidated explicitly after writes, never refetched on a timer
 * or on focus, and never retried (a local IPC failure will not fix itself).
 */
export function createQueryClient(): QueryClient {
  return new QueryClient({
    queryCache: new QueryCache({ onError: handleError }),
    mutationCache: new MutationCache({ onError: handleError }),
    defaultOptions: {
      queries: {
        staleTime: Infinity,
        refetchOnWindowFocus: false,
        refetchOnReconnect: false,
        retry: false,
      },
      mutations: { retry: false },
    },
  });
}

export const queryClient = createQueryClient();
