use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::repositories::account_repository::AccountRepository;

/// Whether the device already has an owner account (decides Register vs Login).
pub struct AccountExistsUseCase {
    accounts: Arc<dyn AccountRepository>,
}

impl AccountExistsUseCase {
    pub fn new(accounts: Arc<dyn AccountRepository>) -> Self {
        Self { accounts }
    }

    pub async fn execute(&self) -> Result<bool, DomainError> {
        self.accounts.exists().await
    }
}
