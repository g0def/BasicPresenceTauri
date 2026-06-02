import { useTranslation } from "react-i18next";

import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { SessionCountdown } from "@/features/auth/presentation/components/session-countdown";
import { LanguageSwitcher } from "@/shared/components/language-switcher";

export function Home() {
  const { user, logout } = useAuth();
  const { t } = useTranslation();

  return (
    <main className="home">
      <header className="home-header">
        <SessionCountdown />
        <LanguageSwitcher />
        <button onClick={() => void logout()}>{t("home.logout")}</button>
      </header>

      <section className="home-welcome">
        <h1>{t("home.greeting", { username: user?.username })}</h1>
        <p>{t("home.intro")}</p>
      </section>
    </main>
  );
}
