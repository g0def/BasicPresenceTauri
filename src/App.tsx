import { useTranslation } from "react-i18next";

import "@/App.css";
import { Home } from "@/features/auth/presentation/components/home";
import { LoginPage } from "@/features/auth/presentation/components/login-page";
import { useAuth } from "@/features/auth/presentation/hooks/use-auth";

/**
 * Auth gate: renders the app only when a valid session exists. Otherwise it
 * shows the register screen (first run) or the login screen.
 */
export default function App() {
  const { isAuthenticated, accountExists } = useAuth();
  const { t } = useTranslation();

  if (isAuthenticated) {
    return <Home />;
  }

  if (accountExists === null) {
    return (
      <main className="auth-card">
        <p>{t("common.loading")}</p>
      </main>
    );
  }

  return <LoginPage />;
}
