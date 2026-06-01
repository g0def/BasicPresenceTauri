import { useState } from "react";
import type { FormEvent } from "react";

import { useAuth } from "@/features/auth/presentation/hooks/use-auth";

export function LoginPage() {
  const { login, register, accountExists, error, isSubmitting, clearError } =
    useAuth();
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
      <h1>{isRegister ? "Créer un compte" : "Connexion"}</h1>
      <p className="auth-subtitle">
        {isRegister
          ? "Première utilisation : créez votre compte propriétaire."
          : "Entrez votre mot de passe pour déverrouiller vos données."}
      </p>

      <form className="auth-form" onSubmit={onSubmit}>
        <label className="auth-field">
          <span>Nom d'utilisateur</span>
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
          <span>Mot de passe</span>
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
            ? "Veuillez patienter…"
            : isRegister
              ? "Créer le compte"
              : "Se connecter"}
        </button>
      </form>
    </main>
  );
}
