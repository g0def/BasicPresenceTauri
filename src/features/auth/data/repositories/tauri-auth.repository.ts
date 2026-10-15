import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import {
  toAuthSession,
  toSessionStatus,
} from "@/features/auth/data/mappers/auth.mapper";
import { toUser } from "@/features/auth/data/mappers/user.mapper";
import type {
  AuthSession,
  Credentials,
  SessionStatus,
} from "@/features/auth/domain/entities/auth-session";
import type { User } from "@/features/auth/domain/entities/user";
import type { AuthRepository } from "@/features/auth/domain/repositories/auth-repository";

/** AuthRepository implementation backed by Tauri IPC commands. */
export class TauriAuthRepository implements AuthRepository {
  async accountExists(): Promise<boolean> {
    return invoke(COMMANDS.accountExists);
  }

  async register({ username, password }: Credentials): Promise<User> {
    const dto = await invoke(COMMANDS.register, {
      username,
      password,
    });
    return toUser(dto);
  }

  async login({ username, password }: Credentials): Promise<AuthSession> {
    const dto = await invoke(COMMANDS.login, {
      username,
      password,
    });
    return toAuthSession(dto);
  }

  async checkSession(token: string): Promise<SessionStatus> {
    const dto = await invoke(COMMANDS.checkSession, {
      token,
    });
    return toSessionStatus(dto);
  }

  async logout(token: string): Promise<void> {
    await invoke(COMMANDS.logout, { token });
  }

  async changePassword(
    currentPassword: string,
    newPassword: string,
  ): Promise<void> {
    await invoke(COMMANDS.changePassword, { currentPassword, newPassword });
  }
}
