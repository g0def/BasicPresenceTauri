import i18n from "@/core/i18n";

/** Error codes the backend can return (`presentation/commands/error.rs`). */
const KNOWN_CODES = [
  "INVALID_CREDENTIALS",
  "ACCOUNT_LOCKED",
  "ACCOUNT_EXISTS",
  "VALIDATION",
  "SESSION_EXPIRED",
  "NOT_FOUND",
  "INTERNAL",
  "EXPORT_FAILED",
  "KEYSTORE_UNRECOVERABLE",
  "VAULT_TAMPERED",
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

function isKnownCode(code: string | undefined): code is AppErrorCode {
  return (KNOWN_CODES as readonly string[]).includes(code ?? "");
}

/** Convert anything thrown by `invoke` into a typed {@link AppError}. */
export function normalizeError(raw: unknown): AppError {
  if (isAppError(raw)) return raw;

  if (raw && typeof raw === "object") {
    const e = raw as BackendError;
    const code: AppErrorCode = isKnownCode(e.code) ? e.code : "UNKNOWN";
    return new AppError(code, e.message ?? i18n.t("errors.generic"));
  }

  if (typeof raw === "string") {
    return new AppError("UNKNOWN", raw);
  }

  return new AppError("UNKNOWN", i18n.t("errors.unknown"));
}
