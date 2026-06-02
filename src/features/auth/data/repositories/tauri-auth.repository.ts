import { COMMANDS } from "@/core/config";
import { invoke } from "@/core/ipc";
import type {
  LoginResponseDto,
  SessionStatusDto,
} from "@/features/auth/data/dto/auth.dto";
import type { UserDto } from "@/features/auth/data/dto/user.dto";
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
    return invoke<boolean>(COMMANDS.accountExists);
  }

  async register({ username, password }: Credentials): Promise<User> {
    const dto = await invoke<UserDto>(COMMANDS.register, {
      username,
      password,
    });
    return toUser(dto);
  }

  async login({ username, password }: Credentials): Promise<AuthSession> {
    const dto = await invoke<LoginResponseDto>(COMMANDS.login, {
      username,
      password,
    });
    return toAuthSession(dto);
  }

  async checkSession(token: string): Promise<SessionStatus> {
    const dto = await invoke<SessionStatusDto>(COMMANDS.checkSession, {
      token,
    });
    return toSessionStatus(dto);
  }

  async logout(token: string): Promise<void> {
    await invoke<void>(COMMANDS.logout, { token });
  }
}
