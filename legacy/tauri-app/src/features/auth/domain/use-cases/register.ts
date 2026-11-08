import type { Credentials } from "@/features/auth/domain/entities/auth-session";
import type { User } from "@/features/auth/domain/entities/user";
import type { AuthRepository } from "@/features/auth/domain/repositories/auth-repository";

export type RegisterUseCase = (credentials: Credentials) => Promise<User>;

export function makeRegisterUseCase(repo: AuthRepository): RegisterUseCase {
  return (credentials) => repo.register(credentials);
}
