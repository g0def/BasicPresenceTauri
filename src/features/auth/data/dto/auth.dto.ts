import type { UserDto } from "@/features/auth/data/dto/user.dto";

export interface LoginResponseDto {
  token: string;
  expiresAt: number;
  user: UserDto;
}

export interface SessionStatusDto {
  valid: boolean;
  remainingMs: number;
}
