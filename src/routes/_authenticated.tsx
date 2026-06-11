import {
  Outlet,
  createFileRoute,
  redirect,
  useNavigate,
} from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

import { SessionCountdown } from "@/features/auth/presentation/components/session-countdown";
import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { CommuteProvider } from "@/features/commute/presentation/providers/commute-provider";
import { PresenceProvider } from "@/features/presence/presentation/providers/presence-provider";
import { EmptyProfileState } from "@/features/profile/presentation/components/empty-profile-state";
import { ProfileBadge } from "@/features/profile/presentation/components/profile-badge";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";
import { ProfileProvider } from "@/features/profile/presentation/providers/profile-provider";
import { TaskPresetProvider } from "@/features/work-hours/presentation/providers/task-preset-provider";
import { Logo } from "@/shared/components/logo";

export const Route = createFileRoute("/_authenticated")({
  beforeLoad: ({ context }) => {
    if (!context.auth.isAuthenticated) {
      throw redirect({ to: "/login" });
    }
  },
  component: AuthenticatedLayout,
});

/**
 * Composition root for the authenticated area: feature providers mount once
 * here and stay alive across page navigations, and session-expired errors
 * from any feature bubble into a logout (the guard above then redirects).
 */
function AuthenticatedLayout() {
  const { logout } = useAuth();

  return (
    <ProfileProvider onSessionExpired={logout}>
      <PresenceProvider onSessionExpired={logout}>
        <CommuteProvider onSessionExpired={logout}>
          <TaskPresetProvider onSessionExpired={logout}>
            <AppShell />
          </TaskPresetProvider>
        </CommuteProvider>
      </PresenceProvider>
    </ProfileProvider>
  );
}

/** App chrome (header + profile gating) shared by every authenticated page. */
function AppShell() {
  const { logout } = useAuth();
  const { profiles, isLoading } = useProfile();
  const { t } = useTranslation();
  const navigate = useNavigate();

  return (
    <main className="flex min-h-screen w-full flex-col p-6">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <Logo className="text-sm" />
        <div className="flex items-center gap-3">
          <SessionCountdown />
          <ProfileBadge
            onLogout={() => void logout()}
            onOpenCommutes={() => void navigate({ to: "/commutes" })}
          />
        </div>
      </header>

      {isLoading ? (
        <section className="flex flex-1 items-center justify-center text-muted-foreground">
          {t("common.loading")}
        </section>
      ) : profiles.length === 0 ? (
        <EmptyProfileState />
      ) : (
        <Outlet />
      )}
    </main>
  );
}
