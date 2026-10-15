import type { ReactNode } from "react";

import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { UpdaterProvider } from "@/features/updater/presentation/providers/updater-provider";

/**
 * Composition glue between two features that do not know each other: before
 * restarting into an installed update, log out so the vault is locked (and its
 * integrity baseline recorded) by the backend.
 */
export function SessionAwareUpdaterProvider({
  children,
}: {
  children: ReactNode;
}) {
  const { logout } = useAuth();
  return <UpdaterProvider beforeRelaunch={logout}>{children}</UpdaterProvider>;
}
