import { createContext } from "react";

import type { Credentials } from "@/features/auth/domain/entities/auth-session";
import type { User } from "@/features/auth/domain/entities/user";

export type AuthStatus = "unauthenticated" | "authenticated";

export interface AuthContextValue {
  status: AuthStatus;
  isAuthenticated: boolean;
  user: User | null;
  /** Milliseconds left before auto-logout (0 when unauthenticated). */
  remainingMs: number;
  /** Whether an owner account exists; `null` while still loading. */
  accountExists: boolean | null;
  error: string | null;
  isSubmitting: boolean;
  /**
   * The last login was refused because the vault failed its at-rest integrity
   * check. The UI offers the recovery path only in that state.
   */
  integrityBlocked: boolean;
  login: (credentials: Credentials) => Promise<void>;
  /** Retry the login accepting the integrity warning (keeps a backup). */
  recoverVault: (credentials: Credentials) => Promise<void>;
  register: (credentials: Credentials) => Promise<void>;
  logout: () => Promise<void>;
  clearError: () => void;
}

// The session token is intentionally NOT exposed here — it stays inside the
// provider so no component can read or leak it.
export const AuthContext = createContext<AuthContextValue | null>(null);
