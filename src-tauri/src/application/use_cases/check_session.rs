use std::sync::Arc;

use crate::application::dto::session_status_dto::SessionStatusDto;
use crate::domain::error::DomainError;
use crate::domain::services::clock::Clock;
use crate::domain::services::session_store::SessionStore;
use crate::domain::services::vault::VaultManager;

/// Report whether a session token is still valid (and how long it has left).
/// An expired token is revoked and the vault is locked as a side effect.
pub struct CheckSessionUseCase {
    sessions: Arc<dyn SessionStore>,
    vault: Arc<dyn VaultManager>,
    clock: Arc<dyn Clock>,
}

impl CheckSessionUseCase {
    pub fn new(
        sessions: Arc<dyn SessionStore>,
        vault: Arc<dyn VaultManager>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            sessions,
            vault,
            clock,
        }
    }

    pub fn execute(&self, token: &str) -> Result<SessionStatusDto, DomainError> {
        let now = self.clock.now_ms();
        match self.sessions.get(token) {
            Some(session) if session.is_valid(now) => Ok(SessionStatusDto {
                valid: true,
                remaining_ms: session.remaining_ms(now),
            }),
            Some(_) => {
                // Expired: revoke and lock the vault.
                self.sessions.remove(token);
                self.vault.close();
                Ok(SessionStatusDto {
                    valid: false,
                    remaining_ms: 0,
                })
            }
            None => Ok(SessionStatusDto {
                valid: false,
                remaining_ms: 0,
            }),
        }
    }
}
