use thiserror::Error;

/// Business errors of the auth domain.
///
/// Low-level details (`Storage`, `Hashing`, `Crypto`, `Token`, `Internal`) are
/// collapsed to a generic error at the presentation boundary so SQL/crypto
/// internals never leak to the WebView.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("account locked")]
    AccountLocked { retry_after_ms: i64 },

    #[error("an account already exists")]
    AccountAlreadyExists,

    #[error("unauthorized: the vault is locked")]
    Unauthorized,

    #[error("profile not found")]
    ProfileNotFound,

    #[error("validation error: {0}")]
    Validation(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("hashing error: {0}")]
    Hashing(String),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("token error: {0}")]
    Token(String),

    #[error("keychain error: {0}")]
    Keychain(String),

    #[error("keystore unrecoverable: the device key is missing or changed")]
    KeystoreUnrecoverable,

    #[error("vault integrity check failed: the encrypted store was modified")]
    VaultTampered,
}
