use std::sync::Arc;

use zeroize::Zeroizing;

use crate::application::services::credentials::{
    self, CredentialGate, verify_password_or_count_failure,
};
use crate::domain::entities::security_event::SecurityEventKind;
use crate::domain::error::DomainError;
use crate::domain::policy::{AuthPolicy, validate_password};
use crate::domain::repositories::account_repository::AccountRepository;
use crate::domain::repositories::security_log::SecurityLog;
use crate::domain::services::clock::Clock;
use crate::domain::services::key_service::KeyService;
use crate::domain::services::password_hasher::PasswordHasher;

/// Change the owner's password.
///
/// Thanks to envelope encryption this never touches the vault: the existing
/// DEK and MAC key are unwrapped with the current password and re-wrapped
/// under a KEK derived from the new one (fresh salt, current parameters).
/// The current password is re-verified and failures count towards the lockout,
/// so an unlocked-but-unattended session cannot be used to guess it.
pub struct ChangePasswordUseCase {
    accounts: Arc<dyn AccountRepository>,
    hasher: Arc<dyn PasswordHasher>,
    keys: Arc<dyn KeyService>,
    clock: Arc<dyn Clock>,
    policy: AuthPolicy,
    gate: CredentialGate,
    log: Arc<dyn SecurityLog>,
}

impl ChangePasswordUseCase {
    pub fn new(
        accounts: Arc<dyn AccountRepository>,
        hasher: Arc<dyn PasswordHasher>,
        keys: Arc<dyn KeyService>,
        clock: Arc<dyn Clock>,
        policy: AuthPolicy,
        gate: CredentialGate,
        log: Arc<dyn SecurityLog>,
    ) -> Self {
        Self {
            accounts,
            hasher,
            keys,
            clock,
            policy,
            gate,
            log,
        }
    }

    pub async fn execute(
        &self,
        current_password: &str,
        new_password: &str,
    ) -> Result<(), DomainError> {
        validate_password(new_password)?;
        if current_password == new_password {
            return Err(DomainError::Validation(
                "the new password must differ from the current one".into(),
            ));
        }
        let _one_attempt_at_a_time = self.gate.lock().await;
        let now = self.clock.now_ms();
        let account = self
            .accounts
            .find_owner()
            .await?
            .ok_or(DomainError::Unauthorized)?;
        let current = Zeroizing::new(current_password.as_bytes().to_vec());
        let new = Zeroizing::new(new_password.as_bytes().to_vec());

        verify_password_or_count_failure(
            self.accounts.as_ref(),
            self.hasher.as_ref(),
            self.log.as_ref(),
            &self.policy,
            &account,
            current.as_slice(),
            now,
        )
        .await?;

        let unlocked = credentials::unlock(
            self.keys.as_ref(),
            &account,
            current.as_slice(),
            &self.policy,
        )
        .await?;
        let mac_key = match unlocked.mac_key {
            Some(key) => key,
            None => self.keys.generate_key()?,
        };
        let key_material = credentials::seal(
            self.keys.as_ref(),
            &account.id,
            new.as_slice(),
            unlocked.dek.as_slice(),
            mac_key.as_slice(),
            &self.policy,
        )
        .await?;
        let password_hash = self.hasher.hash(new.as_slice()).await?;
        self.accounts
            .update_credentials(&account.id, &password_hash, &key_material, now)
            .await?;
        self.accounts
            .reset_failed_attempts(&account.id, now)
            .await?;
        self.log
            .record(SecurityEventKind::PasswordChanged, now)
            .await?;
        Ok(())
    }
}
