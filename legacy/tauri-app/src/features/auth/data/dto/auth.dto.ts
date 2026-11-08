import type { UserDto } from "@/features/auth/data/dto/user.dto";

export interface LoginResponseDto {
  token: string;
  expiresAt: number;
  user: UserDto;
  failedAttemptsSinceLastLogin: number;
  securityEvents: SecurityEventDto[];
}

export interface SecurityEventDto {
  kind: string;
  at: number;
}

export interface SessionStatusDto {
  valid: boolean;
  remainingMs: number;
}
