use async_trait::async_trait;

use crate::domain::entities::account::Account;
use crate::domain::error::DomainError;

/// Persistence contract for the owner account (implemented over the keystore).
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Whether an account already exists (single-owner device).
    async fn exists(&self) -> Result<bool, DomainError>;

    async fn find_by_username(&self, username: &str) -> Result<Option<Account>, DomainError>;

    async fn create(&self, account: &Account) -> Result<(), DomainError>;

    async fn record_failed_attempt(
        &self,
        id: &str,
        failed_attempts: i64,
        locked_until: Option<i64>,
        updated_at: i64,
    ) -> Result<(), DomainError>;

    async fn reset_failed_attempts(&self, id: &str, updated_at: i64) -> Result<(), DomainError>;
}
