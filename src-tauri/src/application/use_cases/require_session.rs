use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::services::clock::Clock;
use crate::domain::services::session_store::SessionStore;
use crate::domain::services::vault::VaultManager;

/// Backend-side session gate, called by every vault-touching Tauri command
/// before its use case runs. The absolute 15-minute expiry must hold even if
/// the frontend never calls `check_session` (e.g. a compromised WebView), so
/// it cannot rely on the frontend's cooperation: when no valid session is
/// left, leftovers are revoked and the vault is locked as a side effect.
pub struct RequireSessionUseCase {
    sessions: Arc<dyn SessionStore>,
    vault: Arc<dyn VaultManager>,
    clock: Arc<dyn Clock>,
}

impl RequireSessionUseCase {
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

    pub fn execute(&self) -> Result<(), DomainError> {
        let now = self.clock.now_ms();
        if self.sessions.list().iter().any(|s| s.is_valid(now)) {
            return Ok(());
        }
        // No valid session: revoke expired leftovers and lock the vault.
        self.sessions.clear();
        self.vault.close();
        Err(DomainError::Unauthorized)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::*;
    use crate::domain::entities::session::Session;
    use crate::infrastructure::session::in_memory_session_store::InMemorySessionStore;

    struct FixedClock(i64);
    impl Clock for FixedClock {
        fn now_ms(&self) -> i64 {
            self.0
        }
    }

    #[derive(Default)]
    struct SpyVault {
        closed: AtomicBool,
    }
    #[async_trait]
    impl VaultManager for SpyVault {
        async fn open(&self, _dek: &[u8], _mac_key: &[u8]) -> Result<(), DomainError> {
            Ok(())
        }
        fn close(&self) {
            self.closed.store(true, Ordering::SeqCst);
        }
    }

    fn use_case(
        sessions: Arc<dyn SessionStore>,
        vault: Arc<SpyVault>,
        now: i64,
    ) -> RequireSessionUseCase {
        RequireSessionUseCase::new(sessions, vault, Arc::new(FixedClock(now)))
    }

    #[test]
    fn passes_while_a_session_is_valid() {
        let sessions: Arc<dyn SessionStore> = Arc::new(InMemorySessionStore::new());
        let vault = Arc::new(SpyVault::default());
        sessions.insert(Session {
            token: "tok".into(),
            expires_at: 1_000,
        });

        assert!(use_case(sessions, vault.clone(), 999).execute().is_ok());
        assert!(!vault.closed.load(Ordering::SeqCst));
    }

    #[test]
    fn expired_session_is_revoked_and_the_vault_locked() {
        let sessions: Arc<dyn SessionStore> = Arc::new(InMemorySessionStore::new());
        let vault = Arc::new(SpyVault::default());
        sessions.insert(Session {
            token: "tok".into(),
            expires_at: 1_000,
        });

        let result = use_case(sessions.clone(), vault.clone(), 1_000).execute();
        assert!(matches!(result, Err(DomainError::Unauthorized)));
        assert!(vault.closed.load(Ordering::SeqCst), "vault must be locked");
        assert!(sessions.list().is_empty(), "expired session revoked");
    }

    #[test]
    fn no_session_at_all_is_unauthorized() {
        let sessions: Arc<dyn SessionStore> = Arc::new(InMemorySessionStore::new());
        let vault = Arc::new(SpyVault::default());

        let result = use_case(sessions, vault, 0).execute();
        assert!(matches!(result, Err(DomainError::Unauthorized)));
    }
}
