use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::services::session_store::SessionStore;
use crate::domain::services::vault::VaultManager;

/// Revoke the session and lock the vault.
///
/// Only the holder of the session token can log out: an unknown token is a
/// no-op, so a caller that does not own the session cannot lock the vault
/// under the feet of the user (and leave a dangling session behind).
pub struct LogoutUseCase {
    sessions: Arc<dyn SessionStore>,
    vault: Arc<dyn VaultManager>,
}

impl LogoutUseCase {
    pub fn new(sessions: Arc<dyn SessionStore>, vault: Arc<dyn VaultManager>) -> Self {
        Self { sessions, vault }
    }

    pub async fn execute(&self, token: &str) -> Result<(), DomainError> {
        if self.sessions.get(token).is_none() {
            return Ok(());
        }
        // Single-owner device: one session at most (login purges the others).
        self.sessions.clear();
        self.vault.close().await;
        Ok(())
    }
}
