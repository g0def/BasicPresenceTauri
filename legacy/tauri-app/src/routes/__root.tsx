import { Suspense, lazy } from "react";
import { Outlet, createRootRouteWithContext } from "@tanstack/react-router";

import type { AuthContextValue } from "@/features/auth/presentation/context/auth-context";

export interface RouterContext {
  auth: AuthContextValue;
}

// Devtools are stripped from production bundles entirely.
const RouterDevtools = import.meta.env.PROD
  ? () => null
  : lazy(() =>
      import("@tanstack/react-router-devtools").then((m) => ({
        default: m.TanStackRouterDevtools,
      })),
    );

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootLayout,
});

function RootLayout() {
  return (
    <>
      <Outlet />
      <Suspense>
        <RouterDevtools />
      </Suspense>
    </>
  );
}
