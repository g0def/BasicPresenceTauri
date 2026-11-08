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
  /** What happened since the previous successful login (shown once). */
  securityNotice: SecurityNotice;
}

/** Security journal entry (`kind` is one of the backend's event kinds). */
export interface SecurityEvent {
  kind: string;
  at: number;
}

export interface SecurityNotice {
  failedAttempts: number;
  events: SecurityEvent[];
}

export interface SessionStatus {
  valid: boolean;
  remainingMs: number;
}
