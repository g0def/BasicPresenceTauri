import { createHashHistory, createRouter } from "@tanstack/react-router";

import { routeTree } from "./routeTree.gen";

/**
 * Module-level singleton (safe under React.StrictMode double-mounts).
 *
 * Hash history because Tauri serves the production build through its custom
 * asset protocol without SPA fallback: a fresh load on a path like
 * /commutes/new would 404 under browser history, while #/commutes/new always
 * resolves to index.html.
 */
export const router = createRouter({
  routeTree,
  history: createHashHistory(),
  // Real auth context is injected per-render via <RouterProvider context>.
  context: { auth: undefined! },
  defaultPreload: "intent",
  scrollRestoration: true,
});

// Register the router for type-safe <Link>, useNavigate, params, etc.
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
