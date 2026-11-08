use std::sync::Arc;

use zeroize::Zeroizing;

use crate::application::dto::login_result_dto::{LoginResultDto, SecurityEventDto};
use crate::application::dto::user_dto::UserDto;
use crate::application::services::credentials::{
    self, CredentialGate, verify_password_or_count_failure,
};
use crate::domain::entities::security_event::SecurityEventKind;
use crate::domain::entities::session::Session;
use crate::domain::error::DomainError;
use crate::domain::policy::{AuthPolicy, MAX_PASSWORD_LEN, MAX_USERNAME_LEN};
use crate::domain::repositories::account_repository::AccountRepository;
use crate::domain::repositories::security_log::SecurityLog;
use crate::domain::services::clock::Clock;
use crate::domain::services::key_service::KeyService;
use crate::domain::services::password_hasher::PasswordHasher;
use crate::domain::services::session_store::SessionStore;
use crate::domain::services::token_generator::TokenGenerator;
use crate::domain::services::vault::{VaultManager, VaultOpenOutcome};

/// Verify credentials (with brute-force protection), unlock the vault, and open
/// a 15-minute session.
pub struct LoginUseCase {
    accounts: Arc<dyn AccountRepository>,
    hasher: Arc<dyn PasswordHasher>,
    keys: Arc<dyn KeyService>,
    tokens: Arc<dyn TokenGenerator>,
    sessions: Arc<dyn SessionStore>,
    vault: Arc<dyn VaultManager>,
    clock: Arc<dyn Clock>,
    policy: AuthPolicy,
    gate: CredentialGate,
    log: Arc<dyn SecurityLog>,
}

impl LoginUseCase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        accounts: Arc<dyn AccountRepository>,
        hasher: Arc<dyn PasswordHasher>,
        keys: Arc<dyn KeyService>,
        tokens: Arc<dyn TokenGenerator>,
        sessions: Arc<dyn SessionStore>,
        vault: Arc<dyn VaultManager>,
        clock: Arc<dyn Clock>,
        policy: AuthPolicy,
        gate: CredentialGate,
        log: Arc<dyn SecurityLog>,
    ) -> Self {
        Self {
            accounts,
            hasher,
            keys,
            tokens,
            sessions,
            vault,
            clock,
            policy,
            gate,
            log,
        }
    }

    pub async fn execute(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LoginResultDto, DomainError> {
        let _one_attempt_at_a_time = self.gate.lock().await;
        let username = username.trim();
        // Inputs no account can have are refused before any hashing work (the
        // length of what the caller sent is no secret).
        if username.chars().count() > MAX_USERNAME_LEN
            || password.chars().count() > MAX_PASSWORD_LEN
        {
            return Err(DomainError::InvalidCredentials);
        }
        let now = self.clock.now_ms();
        let pw = Zeroizing::new(password.as_bytes().to_vec());

        let Some(account) = self.accounts.find_by_username(username).await? else {
            // Equalize timing against username enumeration (hash, discard).
            let _ = self.hasher.hash(pw.as_slice()).await;
            return Err(DomainError::InvalidCredentials);
        };

        verify_password_or_count_failure(
            self.accounts.as_ref(),
            self.hasher.as_ref(),
            self.log.as_ref(),
            &self.policy,
            &account,
            pw.as_slice(),
            now,
        )
        .await?;

        // Success: unwrap the vault keys.
        let unlocked =
            credentials::unlock(self.keys.as_ref(), &account, pw.as_slice(), &self.policy).await?;
        let mac_key = match unlocked.mac_key {
            Some(key) => key,
            // Account predates vault tamper-evidence: mint its MAC key now.
            None => self.keys.generate_key()?,
        };

        // Transparent upgrades, while the password is at hand: stronger KEK
        // parameters / authenticated wrap format, and the auth hash re-done
        // with the current parameters.
        let rehash = self.hasher.needs_rehash(&account.password_hash);
        if rehash || credentials::is_outdated(&account.key_material, &self.policy) {
            let password_hash = if rehash {
                self.hasher.hash(pw.as_slice()).await?
            } else {
                account.password_hash.clone()
            };
            let key_material = credentials::seal(
                self.keys.as_ref(),
                &account.id,
                pw.as_slice(),
                unlocked.dek.as_slice(),
                mac_key.as_slice(),
                &self.policy,
            )
            .await?;
            self.accounts
                .update_credentials(&account.id, &password_hash, &key_material, now)
                .await?;
        }

        let outcome = match self
            .vault
            .open(unlocked.dek.as_slice(), mac_key.as_slice())
            .await
        {
            Ok(outcome) => outcome,
            Err(DomainError::VaultTampered) => {
                let _ = self
                    .log
                    .record(SecurityEventKind::TamperDetected, now)
                    .await;
                return Err(DomainError::VaultTampered);
            }
            Err(e) => return Err(e),
        };
        match outcome {
            VaultOpenOutcome::Clean => {}
            VaultOpenOutcome::RecoveredFromUncleanShutdown => {
                self.log
                    .record(SecurityEventKind::UncleanShutdown, now)
                    .await?;
            }
            VaultOpenOutcome::TamperingTolerated => {
                self.log
                    .record(SecurityEventKind::TamperTolerated, now)
                    .await?;
            }
        }

        // "Since your last visit": events after the previous successful login.
        let previous_login = self
            .log
            .last(SecurityEventKind::Login)
            .await?
            .unwrap_or(i64::MIN);
        let security_events = self
            .log
            .since(previous_login)
            .await?
            .into_iter()
            .filter(|e| e.kind != SecurityEventKind::Login)
            .map(SecurityEventDto::from)
            .collect();
        self.log.record(SecurityEventKind::Login, now).await?;

        // Clear lockout counters and open the session. A new login supersedes
        // any previous session (single-owner device), keeping the invariant
        // "a valid session exists ⇔ the vault is open" exact — check_session
        // and logout close the vault without checking for sibling sessions.
        self.accounts
            .reset_failed_attempts(&account.id, now)
            .await?;
        self.sessions.clear();
        let token = self.tokens.generate()?;
        let expires_at = now + self.policy.session_ttl_ms;
        self.sessions.insert(Session {
            token: token.clone(),
            expires_at,
            last_seen_ms: now,
            idle_timeout_ms: self.policy.idle_timeout_ms,
        });

        Ok(LoginResultDto {
            token,
            expires_at,
            user: UserDto::from(account.to_user()),
            failed_attempts_since_last_login: account.failed_attempts,
            security_events,
        })
    }
}
