import { useEffect } from "react";
import { RouterProvider } from "@tanstack/react-router";

import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { router } from "@/router";

/**
 * Bridges AuthContext into the router. Auth gating itself lives in the route
 * tree (`_authenticated` and `/login` beforeLoad guards); this component only
 * feeds them fresh auth state.
 */
export default function App() {
  const auth = useAuth();

  // Re-run the current route's beforeLoad whenever auth flips — this is what
  // turns login, manual logout, idle timeout and session expiry into the
  // appropriate redirect, without the auth feature knowing about the router.
  useEffect(() => {
    void router.invalidate();
  }, [auth.isAuthenticated]);

  return <RouterProvider router={router} context={{ auth }} />;
}
