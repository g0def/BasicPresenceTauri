import { useTranslation } from "react-i18next";

import { Button } from "@/components/ui/button";
import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { SessionCountdown } from "@/features/auth/presentation/components/session-countdown";
import { Logo } from "@/shared/components/logo";
import { ModeToggle } from "@/shared/components/mode-toggle";
import { LanguageSwitcher } from "@/shared/components/language-switcher";

export function Home() {
  const { user, logout } = useAuth();
  const { t } = useTranslation();

  return (
    <main className="mx-auto max-w-3xl p-6">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <Logo className="text-sm" />
        <div className="flex items-center gap-2">
          <SessionCountdown />
          <LanguageSwitcher />
          <ModeToggle />
          <Button variant="outline" size="sm" onClick={() => void logout()}>
            {t("home.logout")}
          </Button>
        </div>
      </header>

      <section className="mt-12 text-center">
        <h1 className="text-2xl font-semibold">
          {t("home.greeting", { username: user?.username })}
        </h1>
        <p className="mt-2 text-muted-foreground">{t("home.intro")}</p>
      </section>
    </main>
  );
}
