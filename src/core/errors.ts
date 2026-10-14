/**
 * Every `code` the backend can put on the wire, mirroring the match in
 * `src-tauri/src/presentation/commands/error.rs`.
 *
 * Single source of truth: the union type below is derived from this list. They
 * used to be two hand-kept declarations, and drifted — `VAULT_TAMPERED` reached
 * the frontend as `UNKNOWN`, i.e. "Une erreur inconnue", for the one failure the
 * user most needs to understand.
 */
const KNOWN_CODES = [
  "INVALID_CREDENTIALS",
  "ACCOUNT_LOCKED",
  "ACCOUNT_EXISTS",
  "VALIDATION",
  "SESSION_EXPIRED",
  "NOT_FOUND",
  "INTERNAL",
  "EXPORT_FAILED",
  "VAULT_TAMPERED",
  "KEYSTORE_UNRECOVERABLE",
  "SCHEMA_TOO_NEW",
] as const;

export type AppErrorCode = (typeof KNOWN_CODES)[number] | "UNKNOWN";

/** Shape the Rust backend serializes on the `Err` side: `{ code, message }`. */
interface BackendError {
  code?: string;
  message?: string;
}

/** Normalized application error surfaced to the presentation layer. */
export class AppError extends Error {
  readonly code: AppErrorCode;

  constructor(code: AppErrorCode, message: string) {
    super(message);
    this.name = "AppError";
    this.code = code;
  }
}

export function isAppError(value: unknown): value is AppError {
  return value instanceof AppError;
}

/** Convert anything thrown by `invoke` into a typed {@link AppError}. */
export function normalizeError(raw: unknown): AppError {
  if (isAppError(raw)) return raw;

  if (raw && typeof raw === "object") {
    const e = raw as BackendError;
    const code: AppErrorCode = (KNOWN_CODES as readonly string[]).includes(
      e.code ?? "",
    )
      ? (e.code as AppErrorCode)
      : "UNKNOWN";
    return new AppError(code, e.message ?? "Une erreur est survenue");
  }

  if (typeof raw === "string") {
    return new AppError("UNKNOWN", raw);
  }

  return new AppError("UNKNOWN", "Une erreur inconnue est survenue");
}
