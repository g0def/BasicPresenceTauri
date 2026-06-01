export type AppErrorCode =
  | "INVALID_CREDENTIALS"
  | "ACCOUNT_LOCKED"
  | "ACCOUNT_EXISTS"
  | "VALIDATION"
  | "SESSION_EXPIRED"
  | "INTERNAL"
  | "UNKNOWN";

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

const KNOWN_CODES: readonly string[] = [
  "INVALID_CREDENTIALS",
  "ACCOUNT_LOCKED",
  "ACCOUNT_EXISTS",
  "VALIDATION",
  "SESSION_EXPIRED",
  "INTERNAL",
];

/** Convert anything thrown by `invoke` into a typed {@link AppError}. */
export function normalizeError(raw: unknown): AppError {
  if (isAppError(raw)) return raw;

  if (raw && typeof raw === "object") {
    const e = raw as BackendError;
    const code: AppErrorCode = KNOWN_CODES.includes(e.code ?? "")
      ? (e.code as AppErrorCode)
      : "UNKNOWN";
    return new AppError(code, e.message ?? "Une erreur est survenue");
  }

  if (typeof raw === "string") {
    return new AppError("UNKNOWN", raw);
  }

  return new AppError("UNKNOWN", "Une erreur inconnue est survenue");
}
