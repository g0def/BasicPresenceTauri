import { useState } from "react";
import type { FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { Building2, House, Leaf } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { Logo } from "@/shared/components/logo";
import { ModeToggle } from "@/shared/components/mode-toggle";
import { LanguageSwitcher } from "@/shared/components/language-switcher";
import { AuthBackdrop } from "@/features/auth/presentation/components/auth-backdrop";

export function LoginPage() {
  const { login, register, accountExists, error, isSubmitting, clearError } =
    useAuth();
  const { t } = useTranslation();
  const isRegister = accountExists === false;

  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    const credentials = { username, password };
    if (isRegister) {
      void register(credentials);
    } else {
      void login(credentials);
    }
  };

  const disabled = isSubmitting || username.trim() === "" || password === "";

  return (
    <main className="relative flex min-h-screen w-full flex-col items-center justify-center p-6">
      <AuthBackdrop />

      <div className="absolute right-4 top-4 flex items-center gap-1">
        <LanguageSwitcher />
        <ModeToggle />
      </div>

      <div className="flex w-full max-w-sm flex-col gap-5">
        <Card className="shadow-lg">
          <CardHeader className="space-y-2 text-center">
            <Logo className="text-lg" />
            <h1 className="text-xl font-semibold">
              {isRegister ? t("auth.registerTitle") : t("auth.loginTitle")}
            </h1>
            <CardDescription>
              {isRegister
                ? t("auth.registerSubtitle")
                : t("auth.loginSubtitle")}
            </CardDescription>
          </CardHeader>

          <CardContent>
            <form className="flex flex-col gap-4" onSubmit={onSubmit}>
              <div className="flex flex-col gap-2">
                <Label htmlFor="username">{t("auth.username")}</Label>
                <Input
                  id="username"
                  value={username}
                  autoFocus
                  autoComplete="username"
                  onChange={(e) => {
                    setUsername(e.currentTarget.value);
                    clearError();
                  }}
                />
              </div>

              <div className="flex flex-col gap-2">
                <Label htmlFor="password">{t("auth.password")}</Label>
                <Input
                  id="password"
                  type="password"
                  value={password}
                  autoComplete={
                    isRegister ? "new-password" : "current-password"
                  }
                  onChange={(e) => {
                    setPassword(e.currentTarget.value);
                    clearError();
                  }}
                />
              </div>

              {error && (
                <p className="text-sm text-destructive" role="alert">
                  {error}
                </p>
              )}

              <Button type="submit" className="w-full" disabled={disabled}>
                {isSubmitting
                  ? t("auth.submitting")
                  : isRegister
                    ? t("auth.submitRegister")
                    : t("auth.submitLogin")}
              </Button>
            </form>
          </CardContent>
        </Card>

        {/* Récit : accroche, types de présence, et la promesse « local & chiffré » */}
        <div className="flex flex-col items-center gap-3 text-center">
          <p className="text-sm text-muted-foreground">{t("auth.tagline")}</p>

          <ul className="flex items-center justify-center gap-4 text-xs text-muted-foreground">
            <li className="flex items-center gap-1.5">
              <House className="size-3.5 text-success" aria-hidden="true" />
              {t("presence.types.remote")}
            </li>
            <li className="flex items-center gap-1.5">
              <Building2 className="size-3.5 text-primary" aria-hidden="true" />
              {t("presence.types.office")}
            </li>
            <li className="flex items-center gap-1.5">
              <Leaf className="size-3.5 text-success" aria-hidden="true" />
              {t("auth.carbon")}
            </li>
          </ul>

          <p className="text-[11px] uppercase tracking-wide text-muted-foreground/70">
            {t("auth.local")}
          </p>
        </div>
      </div>
    </main>
  );
}
