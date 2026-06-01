import type { SessionStatus } from "@/features/auth/domain/entities/auth-session";
import type { AuthRepository } from "@/features/auth/domain/repositories/auth-repository";

export type CheckSessionUseCase = (token: string) => Promise<SessionStatus>;

export function makeCheckSessionUseCase(
  repo: AuthRepository,
): CheckSessionUseCase {
  return (token) => repo.checkSession(token);
}
