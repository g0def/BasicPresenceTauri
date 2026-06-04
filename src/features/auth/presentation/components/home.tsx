import { useTranslation } from "react-i18next";

import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { SessionCountdown } from "@/features/auth/presentation/components/session-countdown";
import { EmptyProfileState } from "@/features/profile/presentation/components/empty-profile-state";
import { ProfileBadge } from "@/features/profile/presentation/components/profile-badge";
import { useProfile } from "@/features/profile/presentation/hooks/use-profile";
import { Logo } from "@/shared/components/logo";

export function Home() {
  const { user, logout } = useAuth();
  const { profiles, activeProfile, isLoading } = useProfile();
  const { t } = useTranslation();

  return (
    <main className="mx-auto max-w-3xl p-6">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <Logo className="text-sm" />
        <div className="flex items-center gap-3">
          <SessionCountdown />
          <ProfileBadge onLogout={() => void logout()} />
        </div>
      </header>

      {isLoading ? (
        <section className="mt-24 text-center text-muted-foreground">
          {t("common.loading")}
        </section>
      ) : profiles.length === 0 ? (
        <EmptyProfileState />
      ) : (
        <section className="mt-12 text-center">
          <h1 className="text-2xl font-semibold">
            {t("home.greeting", {
              username: activeProfile?.firstName ?? user?.username,
            })}
          </h1>
          <p className="mt-2 text-muted-foreground">{t("home.intro")}</p>
        </section>
      )}
    </main>
  );
}
