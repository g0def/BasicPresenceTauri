use crate::domain::error::DomainError;

/// Port for password hashing/verification (implemented with Argon2id).
pub trait PasswordHasher: Send + Sync {
    /// Hash a password into a self-describing PHC string (salt + params embedded).
    fn hash(&self, password: &[u8]) -> Result<String, DomainError>;

    /// Verify a password against a PHC string. `Ok(false)` on mismatch;
    /// `Err` only on unexpected failures (e.g. malformed hash).
    fn verify(&self, password: &[u8], phc: &str) -> Result<bool, DomainError>;
}
