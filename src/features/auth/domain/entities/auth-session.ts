import type { User } from "@/features/auth/domain/entities/user";

export interface Credentials {
  username: string;
  password: string;
}

/** In-memory session returned by a successful login. Never persisted. */
export interface AuthSession {
  token: string;
  /** Absolute expiry, epoch milliseconds. */
  expiresAt: number;
  user: User;
}

export interface SessionStatus {
  valid: boolean;
  remainingMs: number;
}
