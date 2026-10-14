import { isAppError } from "@/core/errors";

/**
 * Whether a caught error means the backend locked the vault.
 *
 * Every vault-touching command goes through `require_session` in Rust, so any
 * of them can return this once the absolute deadline passes — including the
 * ones that only read. The frontend has to treat it as "the session is over",
 * not as a one-off failure, otherwise the UI keeps showing an authenticated
 * shell over data it can no longer reach.
 *
 * Factored out because the check was open-coded in eight places and missing
 * from three others (export, profile transfer, licence).
 */
export function isSessionExpired(error: unknown): boolean {
  return isAppError(error) && error.code === "SESSION_EXPIRED";
}
