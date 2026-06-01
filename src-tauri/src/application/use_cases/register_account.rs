use std::sync::Arc;

use uuid::Uuid;
use zeroize::Zeroizing;

use crate::application::dto::user_dto::UserDto;
use crate::domain::entities::account::{Account, KeyMaterial};
use crate::domain::error::DomainError;
use crate::domain::repositories::account_repository::AccountRepository;
use crate::domain::services::clock::Clock;
use crate::domain::services::key_service::KeyService;
use crate::domain::services::password_hasher::PasswordHasher;

const MIN_USERNAME_LEN: usize = 3;
const MAX_USERNAME_LEN: usize = 64;
const MIN_PASSWORD_LEN: usize = 8;
const MAX_PASSWORD_LEN: usize = 1024;

/// Create the single device-owner account (first run).
pub struct RegisterAccountUseCase {
    accounts: Arc<dyn AccountRepository>,
    hasher: Arc<dyn PasswordHasher>,
    keys: Arc<dyn KeyService>,
    clock: Arc<dyn Clock>,
}

impl RegisterAccountUseCase {
    pub fn new(
        accounts: Arc<dyn AccountRepository>,
        hasher: Arc<dyn PasswordHasher>,
        keys: Arc<dyn KeyService>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            accounts,
            hasher,
            keys,
            clock,
        }
    }

    pub async fn execute(&self, username: &str, password: &str) -> Result<UserDto, DomainError> {
        let username = username.trim();
        validate_username(username)?;
        validate_password(password)?;

        // Single-owner device: refuse a second account.
        if self.accounts.exists().await? {
            return Err(DomainError::AccountAlreadyExists);
        }

        // Password lives in a buffer that is zeroized on every exit path.
        let pw = Zeroizing::new(password.as_bytes().to_vec());

        let password_hash = self.hasher.hash(pw.as_slice())?;

        // Generate the DEK, derive the password-KEK, wrap the DEK.
        let dek = self.keys.generate_dek()?;
        let kek_salt = self.keys.generate_salt()?;
        let kek = self.keys.derive_kek(pw.as_slice(), &kek_salt)?;
        let wrapped = self.keys.wrap_dek(dek.as_slice(), kek.as_slice())?;

        let now = self.clock.now_ms();
        let account = Account {
            id: Uuid::now_v7().to_string(),
            username: username.to_string(),
            password_hash,
            key_material: KeyMaterial {
                wrapped_dek: wrapped.ciphertext,
                kek_salt,
                dek_nonce: wrapped.nonce,
            },
            failed_attempts: 0,
            locked_until: None,
            created_at: now,
            updated_at: now,
        };
        self.accounts.create(&account).await?;

        Ok(UserDto::from(account.to_user()))
    }
}

fn validate_username(username: &str) -> Result<(), DomainError> {
    let len = username.chars().count();
    if !(MIN_USERNAME_LEN..=MAX_USERNAME_LEN).contains(&len) {
        return Err(DomainError::Validation(format!(
            "username must be {MIN_USERNAME_LEN}-{MAX_USERNAME_LEN} characters"
        )));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), DomainError> {
    let len = password.chars().count();
    if !(MIN_PASSWORD_LEN..=MAX_PASSWORD_LEN).contains(&len) {
        return Err(DomainError::Validation(format!(
            "password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    Ok(())
}
