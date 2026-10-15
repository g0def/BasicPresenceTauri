use async_trait::async_trait;

use crate::domain::entities::account::{Account, KeyMaterial};
use crate::domain::error::DomainError;

/// Persistence contract for the owner account (implemented over the keystore).
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Whether an account already exists (single-owner device).
    async fn exists(&self) -> Result<bool, DomainError>;

    async fn find_by_username(&self, username: &str) -> Result<Option<Account>, DomainError>;

    /// The single device-owner account, if registered.
    async fn find_owner(&self) -> Result<Option<Account>, DomainError>;

    /// Create the owner account. Atomic with respect to the single-owner rule:
    /// fails with [`DomainError::AccountAlreadyExists`] if any account exists,
    /// even under concurrent registrations.
    async fn create(&self, account: &Account) -> Result<(), DomainError>;

    /// Atomically count one more failed login and return the new count.
    async fn register_failed_attempt(&self, id: &str, now: i64) -> Result<i64, DomainError>;

    /// Set (or clear) the lockout deadline.
    async fn set_locked_until(
        &self,
        id: &str,
        locked_until: Option<i64>,
        now: i64,
    ) -> Result<(), DomainError>;

    async fn reset_failed_attempts(&self, id: &str, updated_at: i64) -> Result<(), DomainError>;

    /// Replace the password hash and the whole key material in one write
    /// (password change, key-format upgrade, MAC-key backfill).
    async fn update_credentials(
        &self,
        id: &str,
        password_hash: &str,
        key_material: &KeyMaterial,
        updated_at: i64,
    ) -> Result<(), DomainError>;
}
