import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";

import { IDLE_TIMEOUT_MS } from "@/core/config";
import { isAppError } from "@/core/errors";
import type { Credentials } from "@/features/auth/domain/entities/auth-session";
import type { User } from "@/features/auth/domain/entities/user";
import { makeAccountExistsUseCase } from "@/features/auth/domain/use-cases/account-exists";
import { makeLoginUseCase } from "@/features/auth/domain/use-cases/login";
import { makeLogoutUseCase } from "@/features/auth/domain/use-cases/logout";
import { makeRegisterUseCase } from "@/features/auth/domain/use-cases/register";
import { TauriAuthRepository } from "@/features/auth/data/repositories/tauri-auth.repository";
import {
  AuthContext,
  type AuthContextValue,
  type AuthStatus,
} from "@/features/auth/presentation/context/auth-context";
import { useIdleTimeout } from "@/features/auth/presentation/hooks/use-idle-timeout";
import { useSessionTimer } from "@/features/auth/presentation/hooks/use-session-timer";

// Composition root for the auth feature: wire the repository to the use cases.
// (DI happens here at the edge, never inside the domain.)
const repo = new TauriAuthRepository();
const loginUseCase = makeLoginUseCase(repo);
const logoutUseCase = makeLogoutUseCase(repo);
const registerUseCase = makeRegisterUseCase(repo);
const accountExistsUseCase = makeAccountExistsUseCase(repo);

export function AuthProvider({ children }: { children: ReactNode }) {
  // In-memory ONLY; never written to storage, so an app restart forces re-login.
  const tokenRef = useRef<string | null>(null);
  const [expiresAt, setExpiresAt] = useState<number | null>(null);
  const [user, setUser] = useState<User | null>(null);
  const [status, setStatus] = useState<AuthStatus>("unauthenticated");
  const [accountExists, setAccountExists] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // On startup, ask the backend whether an account already exists.
  useEffect(() => {
    let cancelled = false;
    accountExistsUseCase()
      .then((exists) => {
        if (!cancelled) setAccountExists(exists);
      })
      .catch(() => {
        if (!cancelled) setAccountExists(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const clearSession = useCallback(() => {
    const token = tokenRef.current;
    tokenRef.current = null;
    setExpiresAt(null);
    setUser(null);
    setStatus("unauthenticated");
    if (token) {
      // Best-effort server-side invalidation + vault lock.
      void logoutUseCase(token).catch(() => undefined);
    }
  }, []);

  const remainingMs = useSessionTimer(expiresAt, clearSession);

  // Inactivity logout (only while authenticated); shortens the session when the
  // app is left unattended, on top of the backend's absolute TTL.
  useIdleTimeout(IDLE_TIMEOUT_MS, clearSession, status === "authenticated");

  const login = useCallback(async (credentials: Credentials) => {
    setIsSubmitting(true);
    setError(null);
    try {
      const session = await loginUseCase(credentials);
      tokenRef.current = session.token;
      setExpiresAt(session.expiresAt);
      setUser(session.user);
      setStatus("authenticated");
    } catch (e) {
      setError(isAppError(e) ? e.message : "Échec de la connexion");
    } finally {
      setIsSubmitting(false);
    }
  }, []);

  const register = useCallback(async (credentials: Credentials) => {
    setIsSubmitting(true);
    setError(null);
    try {
      await registerUseCase(credentials);
      // Account created — switch the gate to the login screen.
      setAccountExists(true);
    } catch (e) {
      setError(isAppError(e) ? e.message : "Échec de la création du compte");
    } finally {
      setIsSubmitting(false);
    }
  }, []);

  const logout = useCallback(async () => {
    clearSession();
  }, [clearSession]);

  const clearError = useCallback(() => setError(null), []);

  const value = useMemo<AuthContextValue>(
    () => ({
      status,
      isAuthenticated: status === "authenticated",
      user,
      remainingMs,
      accountExists,
      error,
      isSubmitting,
      login,
      register,
      logout,
      clearError,
    }),
    [
      status,
      user,
      remainingMs,
      accountExists,
      error,
      isSubmitting,
      login,
      register,
      logout,
      clearError,
    ],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
