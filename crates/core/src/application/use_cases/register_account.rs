use std::sync::Arc;

use uuid::Uuid;
use zeroize::Zeroizing;

use crate::application::dto::user_dto::UserDto;
use crate::application::services::credentials;
use crate::domain::entities::account::Account;
use crate::domain::error::DomainError;
use crate::domain::policy::{AuthPolicy, validate_password, validate_username};
use crate::domain::repositories::account_repository::AccountRepository;
use crate::domain::services::clock::Clock;
use crate::domain::services::key_service::KeyService;
use crate::domain::services::password_hasher::PasswordHasher;

/// Create the single device-owner account (first run).
pub struct RegisterAccountUseCase {
    accounts: Arc<dyn AccountRepository>,
    hasher: Arc<dyn PasswordHasher>,
    keys: Arc<dyn KeyService>,
    clock: Arc<dyn Clock>,
    policy: AuthPolicy,
}

impl RegisterAccountUseCase {
    pub fn new(
        accounts: Arc<dyn AccountRepository>,
        hasher: Arc<dyn PasswordHasher>,
        keys: Arc<dyn KeyService>,
        clock: Arc<dyn Clock>,
        policy: AuthPolicy,
    ) -> Self {
        Self {
            accounts,
            hasher,
            keys,
            clock,
            policy,
        }
    }

    pub async fn execute(&self, username: &str, password: &str) -> Result<UserDto, DomainError> {
        let username = username.trim();
        validate_username(username)?;
        validate_password(password)?;

        // Single-owner device: refuse a second account early (the insert below
        // re-checks atomically, for concurrent registrations).
        if self.accounts.exists().await? {
            return Err(DomainError::AccountAlreadyExists);
        }

        // Password lives in a buffer that is zeroized on every exit path.
        let pw = Zeroizing::new(password.as_bytes().to_vec());
        let password_hash = self.hasher.hash(pw.as_slice()).await?;

        // Fresh vault keys (DEK + tamper-evidence MAC key), sealed under the
        // password-derived KEK and bound to this account.
        let id = Uuid::now_v7().to_string();
        let dek = self.keys.generate_key()?;
        let mac_key = self.keys.generate_key()?;
        let key_material = credentials::seal(
            self.keys.as_ref(),
            &id,
            pw.as_slice(),
            dek.as_slice(),
            mac_key.as_slice(),
            &self.policy,
        )
        .await?;

        let now = self.clock.now_ms();
        let account = Account {
            id,
            username: username.to_string(),
            password_hash,
            key_material,
            failed_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: now,
        };
        self.accounts.create(&account).await?;

        Ok(UserDto::from(account.to_user()))
    }
}
