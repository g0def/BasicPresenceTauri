import { useMemo } from "react";
import {
  Outlet,
  createFileRoute,
  redirect,
  useNavigate,
  useLocation,
} from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import { isSameMonth } from "date-fns";
import { Clock, Leaf } from "lucide-react";

import { SessionCountdown } from "@/features/auth/presentation/components/session-countdown";
import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { CommuteProvider } from "@/features/commute/presentation/providers/commute-provider";
import { formatCo2 } from "@/features/commute/presentation/commute-format";
import { usePresence } from "@/features/presence/presentation/hooks/use-presence";
import { PresenceProvider } from "@/features/presence/presentation/providers/presence-provider";
import { EmptyProfileState } from "@/features/profile/presentation/components/empty-profile-state";
import { ProfileBadge } from "@/features/profile/presentation/components/profile-badge";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";
import { ProfileProvider } from "@/features/profile/presentation/providers/profile-provider";
import { formatMinutes } from "@/features/work-hours/presentation/duration-format";
import { TaskPresetProvider } from "@/features/work-hours/presentation/providers/task-preset-provider";
import { useUpdater } from "@/features/updater/presentation/hooks/use-updater";
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
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const { presencesByDay, currentMonth } = usePresence();
  const { checkForUpdates } = useUpdater();

  const lang = i18n.resolvedLanguage ?? "fr";

  // Sum stats for the selected month
  const monthlyStats = useMemo(() => {
    let totalMinutes = 0;
    let totalCo2 = 0;

    for (const presence of presencesByDay.values()) {
      const presenceDate = new Date(presence.day);
      if (isSameMonth(presenceDate, currentMonth)) {
        totalMinutes += presence.workMinutes || 0;
        if (presence.co2Kg != null) {
          totalCo2 += presence.co2Kg;
        }
      }
    }

    return {
      minutes: totalMinutes,
      co2: totalCo2,
    };
  }, [presencesByDay, currentMonth]);

  const formattedHours = formatMinutes(monthlyStats.minutes);
  const formattedCo2 = formatCo2(monthlyStats.co2, lang);

  return (
    <main className="flex min-h-screen w-full flex-col p-6">
      <header className="flex flex-wrap items-center justify-between gap-3 pb-4">
        <Logo className="text-sm" />
        <div className="flex flex-wrap items-center gap-3">
          {pathname === "/" && profiles.length > 0 && (
            <div className="flex items-center gap-2 mr-2">
              <div
                className="flex items-center gap-1.5 rounded-full border border-primary/20 bg-primary/5 px-3 py-1 text-xs font-semibold text-primary"
                title={t("summary.hoursWorked", { hours: formattedHours })}
              >
                <Clock className="size-3.5" />
                <span>
                  {t("summary.hoursWorked", { hours: formattedHours })}
                </span>
              </div>
              <div
                className="flex items-center gap-1.5 rounded-full border border-success/20 bg-success/5 px-3 py-1 text-xs font-semibold text-success"
                title={t("summary.co2Emitted", { co2: formattedCo2 })}
              >
                <Leaf className="size-3.5" />
                <span>{t("summary.co2Emitted", { co2: formattedCo2 })}</span>
              </div>
            </div>
          )}
          <SessionCountdown />
          <ProfileBadge
            onLogout={() => void logout()}
            onOpenCommutes={() => void navigate({ to: "/commutes" })}
            onCheckUpdates={() => void checkForUpdates(true)}
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
