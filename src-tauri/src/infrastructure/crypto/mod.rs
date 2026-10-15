pub mod argon2_hasher;
pub mod device_key;
pub mod key_service;
pub mod token_generator;

use crate::domain::error::DomainError;

/// Run CPU-heavy crypto (Argon2) on the blocking thread pool, so a login never
/// stalls the async executor that serves every other command.
pub(crate) async fn run_blocking<T, F>(work: F) -> Result<T, DomainError>
where
    F: FnOnce() -> Result<T, DomainError> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|e| DomainError::Crypto(format!("blocking task failed: {e}")))?
}
