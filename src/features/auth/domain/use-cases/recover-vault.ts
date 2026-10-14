import type {
  AuthSession,
  Credentials,
} from "@/features/auth/domain/entities/auth-session";
import type { AuthRepository } from "@/features/auth/domain/repositories/auth-repository";

export type RecoverVaultUseCase = (
  credentials: Credentials,
) => Promise<AuthSession>;

/**
 * Log in while accepting a failed at-rest integrity check.
 *
 * Only meaningful once `login` has returned `VAULT_TAMPERED`. That verdict has
 * two causes the file cannot tell apart — a real modification, or a crash at the
 * wrong moment (power cut during a migration, machine killed mid-write) — so
 * refusing to open must not be a dead end, or an accident would cost the owner
 * all their data. The backend keeps a copy of the suspect file before
 * re-baselining it.
 */
export function makeRecoverVaultUseCase(
  repo: AuthRepository,
): RecoverVaultUseCase {
  return (credentials) => repo.recoverVault(credentials);
}
