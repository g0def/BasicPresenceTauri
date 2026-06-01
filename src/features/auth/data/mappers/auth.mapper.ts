import type {
  LoginResponseDto,
  SessionStatusDto,
} from "@/features/auth/data/dto/auth.dto";
import { toUser } from "@/features/auth/data/mappers/user.mapper";
import type {
  AuthSession,
  SessionStatus,
} from "@/features/auth/domain/entities/auth-session";

export function toAuthSession(dto: LoginResponseDto): AuthSession {
  return {
    token: dto.token,
    expiresAt: dto.expiresAt,
    user: toUser(dto.user),
  };
}

export function toSessionStatus(dto: SessionStatusDto): SessionStatus {
  return {
    valid: dto.valid,
    remainingMs: dto.remainingMs,
  };
}
