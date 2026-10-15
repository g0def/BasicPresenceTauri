import type {
  AuthSession,
  Credentials,
  SessionStatus,
} from "@/features/auth/domain/entities/auth-session";
import type { User } from "@/features/auth/domain/entities/user";

/** Contract the presentation layer depends on; implemented in the data layer. */
export interface AuthRepository {
  accountExists(): Promise<boolean>;
  register(credentials: Credentials): Promise<User>;
  login(credentials: Credentials): Promise<AuthSession>;
  checkSession(token: string): Promise<SessionStatus>;
  logout(token: string): Promise<void>;
  /** Re-wrap the vault keys under a new password (requires an open session;
   *  the current password is re-verified). */
  changePassword(currentPassword: string, newPassword: string): Promise<void>;
}
