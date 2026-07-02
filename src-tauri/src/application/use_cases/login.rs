use std::sync::Arc;

use zeroize::Zeroizing;

use crate::application::dto::login_result_dto::LoginResultDto;
use crate::application::dto::user_dto::UserDto;
use crate::domain::entities::session::Session;
use crate::domain::error::DomainError;
use crate::domain::repositories::account_repository::AccountRepository;
use crate::domain::services::clock::Clock;
use crate::domain::services::key_service::{KeyService, WrappedKey};
use crate::domain::services::password_hasher::PasswordHasher;
use crate::domain::services::session_store::SessionStore;
use crate::domain::services::token_generator::TokenGenerator;
use crate::domain::services::vault::VaultManager;
use crate::infrastructure::config::AuthPolicy;

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
        }
    }

    pub async fn execute(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LoginResultDto, DomainError> {
        let username = username.trim();
        let now = self.clock.now_ms();
        let pw = Zeroizing::new(password.as_bytes().to_vec());

        let Some(account) = self.accounts.find_by_username(username).await? else {
            // Equalize timing against username enumeration (hash, discard).
            let _ = self.hasher.hash(pw.as_slice());
            return Err(DomainError::InvalidCredentials);
        };

        // Locked out?
        if let Some(locked_until) = account.locked_until {
            if locked_until > now {
                return Err(DomainError::AccountLocked {
                    retry_after_ms: locked_until - now,
                });
            }
        }

        // Verify the password.
        if !self.hasher.verify(pw.as_slice(), &account.password_hash)? {
            let attempts = account.failed_attempts + 1;
            let locked_until = if attempts >= self.policy.max_attempts {
                Some(now + self.policy.lockout_ms)
            } else {
                None
            };
            self.accounts
                .record_failed_attempt(&account.id, attempts, locked_until, now)
                .await?;
            return Err(DomainError::InvalidCredentials);
        }

        // Success: derive the KEK, unwrap the DEK, unlock the vault.
        let kek = self
            .keys
            .derive_kek(pw.as_slice(), &account.key_material.kek_salt)?;
        let wrapped = WrappedKey {
            ciphertext: account.key_material.wrapped_dek.clone(),
            nonce: account.key_material.dek_nonce.clone(),
        };
        let dek = self.keys.unwrap_dek(&wrapped, kek.as_slice())?;

        // Resolve the vault MAC key (tamper-evidence), backfilling it once for
        // accounts created before the feature shipped.
        let mac_key = match (
            &account.key_material.wrapped_mac_key,
            &account.key_material.mac_key_nonce,
        ) {
            (Some(ciphertext), Some(nonce)) => {
                let wrapped_mac = WrappedKey {
                    ciphertext: ciphertext.clone(),
                    nonce: nonce.clone(),
                };
                self.keys.unwrap_dek(&wrapped_mac, kek.as_slice())?
            }
            _ => {
                let new_mac = self.keys.generate_dek()?;
                let wrapped_mac = self.keys.wrap_dek(new_mac.as_slice(), kek.as_slice())?;
                self.accounts
                    .set_mac_key(
                        &account.id,
                        &wrapped_mac.ciphertext,
                        &wrapped_mac.nonce,
                        now,
                    )
                    .await?;
                new_mac
            }
        };

        self.vault.open(dek.as_slice(), mac_key.as_slice()).await?;

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
        });

        Ok(LoginResultDto {
            token,
            expires_at,
            user: UserDto::from(account.to_user()),
        })
    }
}
