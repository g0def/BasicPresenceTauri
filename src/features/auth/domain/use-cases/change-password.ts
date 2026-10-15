import type { AuthRepository } from "@/features/auth/domain/repositories/auth-repository";

/** Same rule as the backend (NIST SP 800-63B: length only). */
export const MIN_PASSWORD_LENGTH = 8;

export type ChangePasswordUseCase = (
  currentPassword: string,
  newPassword: string,
) => Promise<void>;

export function makeChangePasswordUseCase(
  repo: AuthRepository,
): ChangePasswordUseCase {
  return (currentPassword, newPassword) =>
    repo.changePassword(currentPassword, newPassword);
}
