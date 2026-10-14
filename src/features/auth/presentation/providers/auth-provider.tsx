import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";

import { IDLE_TIMEOUT_MS } from "@/core/config";
import { isAppError } from "@/core/errors";
import type {
  AuthSession,
  Credentials,
} from "@/features/auth/domain/entities/auth-session";
import type { User } from "@/features/auth/domain/entities/user";
import { makeAccountExistsUseCase } from "@/features/auth/domain/use-cases/account-exists";
import { makeCheckSessionUseCase } from "@/features/auth/domain/use-cases/check-session";
import { makeLoginUseCase } from "@/features/auth/domain/use-cases/login";
import { makeRecoverVaultUseCase } from "@/features/auth/domain/use-cases/recover-vault";
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
const recoverVaultUseCase = makeRecoverVaultUseCase(repo);
const accountExistsUseCase = makeAccountExistsUseCase(repo);
const checkSessionUseCase = makeCheckSessionUseCase(repo);

export function AuthProvider({ children }: { children: ReactNode }) {
  // In-memory ONLY; never written to storage, so an app restart forces re-login.
  const tokenRef = useRef<string | null>(null);
  const [expiresAt, setExpiresAt] = useState<number | null>(null);
  const [user, setUser] = useState<User | null>(null);
  const [status, setStatus] = useState<AuthStatus>("unauthenticated");
  const [accountExists, setAccountExists] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  // Set when the backend refuses to open the vault over a failed integrity
  // check. Drives the recovery prompt on the login screen — without it, a power
  // cut during a migration would look like a permanent, unexplained lockout.
  const [integrityBlocked, setIntegrityBlocked] = useState(false);

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

  // Re-sync with the backend when the window comes back to the foreground.
  //
  // The countdown runs on the wall clock, which drifts across a suspend and can
  // be changed outright; the backend holds the authoritative deadline. Asking it
  // on resume means a session that died while the machine was asleep is
  // reflected immediately, instead of showing time that no longer exists and
  // failing on the next command.
  useEffect(() => {
    if (status !== "authenticated") return;

    const resync = () => {
      if (document.visibilityState !== "visible") return;
      const token = tokenRef.current;
      if (!token) return;
      void checkSessionUseCase(token)
        .then((session) => {
          if (tokenRef.current !== token) return; // logged out meanwhile
          if (session.valid) {
            setExpiresAt(Date.now() + session.remainingMs);
          } else {
            clearSession();
          }
        })
        .catch(() => undefined); // offline-local app: a failure here is not fatal
    };

    document.addEventListener("visibilitychange", resync);
    return () => document.removeEventListener("visibilitychange", resync);
  }, [status, clearSession]);

  // Inactivity logout (only while authenticated); shortens the session when the
  // app is left unattended, on top of the backend's absolute TTL.
  useIdleTimeout(IDLE_TIMEOUT_MS, clearSession, status === "authenticated");

  const applySession = useCallback((session: AuthSession) => {
    tokenRef.current = session.token;
    setExpiresAt(session.expiresAt);
    setUser(session.user);
    setStatus("authenticated");
    setIntegrityBlocked(false);
  }, []);

  const login = useCallback(
    async (credentials: Credentials) => {
      setIsSubmitting(true);
      setError(null);
      try {
        applySession(await loginUseCase(credentials));
      } catch (e) {
        if (isAppError(e) && e.code === "VAULT_TAMPERED") {
          setIntegrityBlocked(true);
        }
        setError(isAppError(e) ? e.message : "Échec de la connexion");
      } finally {
        setIsSubmitting(false);
      }
    },
    [applySession],
  );

  /** Retry the login, accepting the integrity warning (see recover-vault). */
  const recoverVault = useCallback(
    async (credentials: Credentials) => {
      setIsSubmitting(true);
      setError(null);
      try {
        applySession(await recoverVaultUseCase(credentials));
      } catch (e) {
        setError(isAppError(e) ? e.message : "Échec de la récupération");
      } finally {
        setIsSubmitting(false);
      }
    },
    [applySession],
  );

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
      integrityBlocked,
      login,
      recoverVault,
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
      integrityBlocked,
      login,
      recoverVault,
      register,
      logout,
      clearError,
    ],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
