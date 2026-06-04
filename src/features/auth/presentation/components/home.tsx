import { useTranslation } from "react-i18next";

import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { SessionCountdown } from "@/features/auth/presentation/components/session-countdown";
import { PresenceCalendar } from "@/features/presence/presentation/components/presence-calendar";
import { EmptyProfileState } from "@/features/profile/presentation/components/empty-profile-state";
import { ProfileBadge } from "@/features/profile/presentation/components/profile-badge";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";
import { Logo } from "@/shared/components/logo";

export function Home() {
  const { logout } = useAuth();
  const { profiles, isLoading } = useProfile();
  const { t } = useTranslation();

  return (
    <main className="flex min-h-screen w-full flex-col p-6">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <Logo className="text-sm" />
        <div className="flex items-center gap-3">
          <SessionCountdown />
          <ProfileBadge onLogout={() => void logout()} />
        </div>
      </header>

      {isLoading ? (
        <section className="flex flex-1 items-center justify-center text-muted-foreground">
          {t("common.loading")}
        </section>
      ) : profiles.length === 0 ? (
        <EmptyProfileState />
      ) : (
        <PresenceCalendar />
      )}
    </main>
  );
}
