import { createFileRoute, redirect } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

import { LoginPage } from "@/features/auth/presentation/components/login-page";
import { useAuth } from "@/features/auth/presentation/hooks/use-auth";

export const Route = createFileRoute("/login")({
  beforeLoad: ({ context }) => {
    if (context.auth.isAuthenticated) {
      throw redirect({ to: "/" });
    }
  },
  component: LoginRoute,
});

function LoginRoute() {
  const { accountExists } = useAuth();
  const { t } = useTranslation();

  // Still checking whether an owner account exists (first paint after startup).
  if (accountExists === null) {
    return (
      <main className="flex min-h-screen items-center justify-center p-6">
        <p className="text-muted-foreground">{t("common.loading")}</p>
      </main>
    );
  }

  return <LoginPage />;
}
