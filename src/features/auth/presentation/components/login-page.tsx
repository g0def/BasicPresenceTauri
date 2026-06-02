import { useState } from "react";
import type { FormEvent } from "react";
import { useTranslation } from "react-i18next";

import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
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
    <main className="auth-card">
      <div className="auth-card-toolbar">
        <LanguageSwitcher />
      </div>
      <h1>{isRegister ? t("auth.registerTitle") : t("auth.loginTitle")}</h1>
      <p className="auth-subtitle">
        {isRegister ? t("auth.registerSubtitle") : t("auth.loginSubtitle")}
      </p>

      <form className="auth-form" onSubmit={onSubmit}>
        <label className="auth-field">
          <span>{t("auth.username")}</span>
          <input
            value={username}
            autoFocus
            autoComplete="username"
            onChange={(e) => {
              setUsername(e.currentTarget.value);
              clearError();
            }}
          />
        </label>

        <label className="auth-field">
          <span>{t("auth.password")}</span>
          <input
            type="password"
            value={password}
            autoComplete={isRegister ? "new-password" : "current-password"}
            onChange={(e) => {
              setPassword(e.currentTarget.value);
              clearError();
            }}
          />
        </label>

        {error && (
          <p className="auth-error" role="alert">
            {error}
          </p>
        )}

        <button type="submit" disabled={disabled}>
          {isSubmitting
            ? t("auth.submitting")
            : isRegister
              ? t("auth.submitRegister")
              : t("auth.submitLogin")}
        </button>
      </form>
    </main>
  );
}
