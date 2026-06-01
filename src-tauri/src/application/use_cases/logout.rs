use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::services::session_store::SessionStore;
use crate::domain::services::vault::VaultManager;

/// Revoke the session and lock the vault.
pub struct LogoutUseCase {
    sessions: Arc<dyn SessionStore>,
    vault: Arc<dyn VaultManager>,
}

impl LogoutUseCase {
    pub fn new(sessions: Arc<dyn SessionStore>, vault: Arc<dyn VaultManager>) -> Self {
        Self { sessions, vault }
    }

    pub fn execute(&self, token: &str) -> Result<(), DomainError> {
        self.sessions.remove(token);
        self.vault.close();
        Ok(())
    }
}
