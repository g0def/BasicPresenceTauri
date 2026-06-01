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
  login: (credentials: Credentials) => Promise<void>;
  register: (credentials: Credentials) => Promise<void>;
  logout: () => Promise<void>;
  clearError: () => void;
}

// The session token is intentionally NOT exposed here — it stays inside the
// provider so no component can read or leak it.
export const AuthContext = createContext<AuthContextValue | null>(null);
