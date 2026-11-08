use async_trait::async_trait;

use crate::domain::error::DomainError;

/// Port for password hashing/verification (implemented with Argon2id).
///
/// Async because hashing is deliberately expensive (tens of MiB, hundreds of
/// ms): implementations must run it off the async executor.
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    /// Hash a password into a self-describing PHC string (salt + params embedded).
    async fn hash(&self, password: &[u8]) -> Result<String, DomainError>;

    /// Verify a password against a PHC string. `Ok(false)` on mismatch;
    /// `Err` on unexpected failures (malformed hash, or parameters outside the
    /// accepted bounds — a tampered PHC string must not cause a DoS).
    async fn verify(&self, password: &[u8], phc: &str) -> Result<bool, DomainError>;

    /// Whether a PHC string was produced with other parameters than the
    /// current policy (it should then be re-hashed at the next login).
    fn needs_rehash(&self, phc: &str) -> bool;
}
