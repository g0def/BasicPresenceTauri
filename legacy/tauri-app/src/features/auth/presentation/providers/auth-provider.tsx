import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";

import { useTranslation } from "react-i18next";

import { IDLE_TIMEOUT_MS } from "@/core/config";
import { useContainer } from "@/core/di";
import { isAppError } from "@/core/errors";
import type {
  Credentials,
  SecurityNotice,
} from "@/features/auth/domain/entities/auth-session";
import type { User } from "@/features/auth/domain/entities/user";
import { makeAccountExistsUseCase } from "@/features/auth/domain/use-cases/account-exists";
import { makeCheckSessionUseCase } from "@/features/auth/domain/use-cases/check-session";
import { makeLoginUseCase } from "@/features/auth/domain/use-cases/login";
import { makeLogoutUseCase } from "@/features/auth/domain/use-cases/logout";
import { makeRegisterUseCase } from "@/features/auth/domain/use-cases/register";
import {
  AuthContext,
  type AuthContextValue,
  type AuthStatus,
} from "@/features/auth/presentation/context/auth-context";
import { useIdleTimeout } from "@/features/auth/presentation/hooks/use-idle-timeout";
import { useSessionTimer } from "@/features/auth/presentation/hooks/use-session-timer";

export function AuthProvider({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  // Use cases wired to the injected repository (composition root: app/container).
  const { authRepository } = useContainer();
  const {
    loginUseCase,
    logoutUseCase,
    registerUseCase,
    accountExistsUseCase,
    checkSessionUseCase,
  } = useMemo(
    () => ({
      loginUseCase: makeLoginUseCase(authRepository),
      logoutUseCase: makeLogoutUseCase(authRepository),
      registerUseCase: makeRegisterUseCase(authRepository),
      accountExistsUseCase: makeAccountExistsUseCase(authRepository),
      checkSessionUseCase: makeCheckSessionUseCase(authRepository),
    }),
    [authRepository],
  );

  // In-memory ONLY; never written to storage, so an app restart forces re-login.
  const tokenRef = useRef<string | null>(null);
  const [expiresAt, setExpiresAt] = useState<number | null>(null);
  const [user, setUser] = useState<User | null>(null);
  const [status, setStatus] = useState<AuthStatus>("unauthenticated");
  const [accountExists, setAccountExists] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [securityNotice, setSecurityNotice] = useState<SecurityNotice | null>(
    null,
  );

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
  }, [accountExistsUseCase]);

  // Drop the session locally at once, then have the backend revoke it and lock
  // the vault. The returned promise settles when the backend has answered —
  // awaited by callers that must not proceed before the vault is locked
  // (restart into an update). A failed backend logout is not fatal: the
  // backend's own idle/absolute expiry locks the vault anyway.
  const clearSession = useCallback(async (): Promise<void> => {
    const token = tokenRef.current;
    tokenRef.current = null;
    setExpiresAt(null);
    setUser(null);
    setSecurityNotice(null);
    setStatus("unauthenticated");
    if (token) {
      await logoutUseCase(token).catch(() => undefined);
    }
  }, [logoutUseCase]);

  const expire = useCallback(() => void clearSession(), [clearSession]);
  const remainingMs = useSessionTimer(expiresAt, expire);

  // While the user is active in the UI, tell the backend (throttled): it runs
  // its own idle timer and would otherwise lock the vault under an active user
  // who happens not to trigger any backend call. A session the backend no
  // longer knows ends here too.
  const reportActivity = useCallback(() => {
    const token = tokenRef.current;
    if (!token) return;
    checkSessionUseCase(token)
      .then((s) => {
        if (!s.valid) void clearSession();
      })
      .catch(() => undefined);
  }, [clearSession, checkSessionUseCase]);

  // Inactivity logout (only while authenticated); shortens the session when the
  // app is left unattended, on top of the backend's absolute TTL.
  useIdleTimeout(
    IDLE_TIMEOUT_MS,
    expire,
    status === "authenticated",
    reportActivity,
  );

  const login = useCallback(
    async (credentials: Credentials) => {
      setIsSubmitting(true);
      setError(null);
      try {
        const session = await loginUseCase(credentials);
        tokenRef.current = session.token;
        setExpiresAt(session.expiresAt);
        setUser(session.user);
        const notice = session.securityNotice;
        setSecurityNotice(
          notice.failedAttempts > 0 || notice.events.length > 0 ? notice : null,
        );
        setStatus("authenticated");
      } catch (e) {
        setError(isAppError(e) ? e.message : t("errors.loginFailed"));
      } finally {
        setIsSubmitting(false);
      }
    },
    [loginUseCase, t],
  );

  const register = useCallback(
    async (credentials: Credentials) => {
      setIsSubmitting(true);
      setError(null);
      try {
        await registerUseCase(credentials);
        // Account created — switch the gate to the login screen.
        setAccountExists(true);
      } catch (e) {
        setError(isAppError(e) ? e.message : t("errors.registerFailed"));
      } finally {
        setIsSubmitting(false);
      }
    },
    [registerUseCase, t],
  );

  const logout = useCallback(() => clearSession(), [clearSession]);

  const clearError = useCallback(() => setError(null), []);
  const dismissSecurityNotice = useCallback(() => setSecurityNotice(null), []);

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
      securityNotice,
      dismissSecurityNotice,
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
      securityNotice,
      dismissSecurityNotice,
    ],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
