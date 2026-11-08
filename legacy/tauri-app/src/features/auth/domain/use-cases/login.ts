import type {
  AuthSession,
  Credentials,
} from "@/features/auth/domain/entities/auth-session";
import type { AuthRepository } from "@/features/auth/domain/repositories/auth-repository";

export type LoginUseCase = (credentials: Credentials) => Promise<AuthSession>;

export function makeLoginUseCase(repo: AuthRepository): LoginUseCase {
  return (credentials) => repo.login(credentials);
}
