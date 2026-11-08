// Ports this feature takes from the DI container (see core/di).
import type { AuthRepository } from "@/features/auth/domain/repositories/auth-repository";

declare module "@/core/di" {
  interface Container {
    authRepository: AuthRepository;
  }
}
