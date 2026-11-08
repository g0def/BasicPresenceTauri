import type { AuthRepository } from "@/features/auth/domain/repositories/auth-repository";

export type LogoutUseCase = (token: string) => Promise<void>;

export function makeLogoutUseCase(repo: AuthRepository): LogoutUseCase {
  return (token) => repo.logout(token);
}
