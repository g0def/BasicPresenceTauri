import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";

import { setSessionExpiredHandler } from "@/app/query-client";
import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { router } from "@/router";

/**
 * Bridges AuthContext into the router. Auth gating itself lives in the route
 * tree (`_authenticated` and `/login` beforeLoad guards); this component only
 * feeds them fresh auth state.
 */
export default function App() {
  const auth = useAuth();
  const queryClient = useQueryClient();

  // A SESSION_EXPIRED from any query/mutation logs out, once, centrally.
  const { logout } = auth;
  useEffect(() => {
    setSessionExpiredHandler(() => void logout());
    return () => setSessionExpiredHandler(null);
  }, [logout]);

  // Leaving the session drops every cached vault read from memory: the data is
  // confidential and must not outlive the session that unlocked it.
  useEffect(() => {
    if (!auth.isAuthenticated) queryClient.clear();
  }, [auth.isAuthenticated, queryClient]);

  // Re-run the current route's beforeLoad whenever auth flips — this is what
  // turns login, manual logout, idle timeout and session expiry into the
  // appropriate redirect, without the auth feature knowing about the router.
  useEffect(() => {
    void router.invalidate();
  }, [auth.isAuthenticated]);

  return <RouterProvider router={router} context={{ auth }} />;
}
