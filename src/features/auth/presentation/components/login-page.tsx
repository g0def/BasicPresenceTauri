import { useState } from "react";
import type { FormEvent } from "react";
import { useTranslation } from "react-i18next";

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
    <main className="mx-auto flex min-h-screen w-full max-w-sm flex-col justify-center gap-4 p-6">
      <div className="flex items-center justify-end gap-1">
        <LanguageSwitcher />
        <ModeToggle />
      </div>

      <Card>
        <CardHeader className="space-y-2 text-center">
          <Logo className="text-lg" />
          <h1 className="text-xl font-semibold">
            {isRegister ? t("auth.registerTitle") : t("auth.loginTitle")}
          </h1>
          <CardDescription>
            {isRegister ? t("auth.registerSubtitle") : t("auth.loginSubtitle")}
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
                autoComplete={isRegister ? "new-password" : "current-password"}
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
    </main>
  );
}
