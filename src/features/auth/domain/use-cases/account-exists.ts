import type { AuthRepository } from "@/features/auth/domain/repositories/auth-repository";

export type AccountExistsUseCase = () => Promise<boolean>;

export function makeAccountExistsUseCase(
  repo: AuthRepository,
): AccountExistsUseCase {
  return () => repo.accountExists();
}
