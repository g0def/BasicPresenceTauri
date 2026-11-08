//! Envelope-encryption bookkeeping shared by register, login and
//! change-password: unlocking the vault keys with a password, and sealing them
//! under a (new) password in the current format.

use std::sync::Arc;

use zeroize::Zeroizing;

use crate::domain::entities::account::{Account, KeyMaterial};
use crate::domain::entities::security_event::SecurityEventKind;
use crate::domain::error::DomainError;
use crate::domain::policy::AuthPolicy;
use crate::domain::repositories::account_repository::AccountRepository;
use crate::domain::repositories::security_log::SecurityLog;
use crate::domain::services::key_service::{
    CURRENT_WRAP_VERSION, KeyBinding, KeyRole, KeyService, WrappedKey,
};
use crate::domain::services::password_hasher::PasswordHasher;

/// Serializes password checks (login, change-password): attempts are evaluated
/// one at a time, so parallel guesses cannot all slip past the lockout check
/// before the failure counter catches up.
pub type CredentialGate = Arc<tokio::sync::Mutex<()>>;

/// Check that `password` matches `account`, counting a failure towards the
/// (progressive) lockout. Shared with change-password.
pub async fn verify_password_or_count_failure(
    accounts: &dyn AccountRepository,
    hasher: &dyn PasswordHasher,
    log: &dyn SecurityLog,
    policy: &AuthPolicy,
    account: &Account,
    password: &[u8],
    now: i64,
) -> Result<(), DomainError> {
    if let Some(locked_until) = account.locked_until
        && locked_until > now
    {
        return Err(DomainError::AccountLocked {
            retry_after_ms: locked_until - now,
        });
    }
    if hasher.verify(password, &account.password_hash).await? {
        return Ok(());
    }
    let attempts = accounts.register_failed_attempt(&account.id, now).await?;
    if let Some(lockout_ms) = policy.lockout_for(attempts) {
        accounts
            .set_locked_until(&account.id, Some(now + lockout_ms), now)
            .await?;
        log.record(SecurityEventKind::AccountLocked, now).await?;
    }
    Err(DomainError::InvalidCredentials)
}

/// The vault keys, unwrapped.
pub struct VaultKeys {
    pub dek: Zeroizing<[u8; 32]>,
    /// `None` for accounts created before vault tamper-evidence existed.
    pub mac_key: Option<Zeroizing<[u8; 32]>>,
}

/// Unwrap the vault keys of `account` with `password`.
pub async fn unlock(
    keys: &dyn KeyService,
    account: &Account,
    password: &[u8],
    policy: &AuthPolicy,
) -> Result<VaultKeys, DomainError> {
    let km = &account.key_material;
    let params = km.kdf.unwrap_or(policy.legacy_kek_kdf);
    let kek = keys.derive_kek(password, &km.kek_salt, params).await?;
    let binding = |role| KeyBinding {
        version: km.key_version,
        role,
        account_id: &account.id,
    };
    let dek = keys.unwrap_key(
        &WrappedKey {
            ciphertext: km.wrapped_dek.clone(),
            nonce: km.dek_nonce.clone(),
        },
        kek.as_slice(),
        binding(KeyRole::Dek),
    )?;
    let mac_key = match (&km.wrapped_mac_key, &km.mac_key_nonce) {
        (Some(ciphertext), Some(nonce)) => Some(keys.unwrap_key(
            &WrappedKey {
                ciphertext: ciphertext.clone(),
                nonce: nonce.clone(),
            },
            kek.as_slice(),
            binding(KeyRole::VaultMac),
        )?),
        _ => None,
    };
    Ok(VaultKeys { dek, mac_key })
}

/// Wrap `dek` and `mac_key` under a KEK freshly derived from `password` (new
/// salt, current KDF parameters, current wrap format).
pub async fn seal(
    keys: &dyn KeyService,
    account_id: &str,
    password: &[u8],
    dek: &[u8],
    mac_key: &[u8],
    policy: &AuthPolicy,
) -> Result<KeyMaterial, DomainError> {
    let kek_salt = keys.generate_salt()?;
    let kek = keys.derive_kek(password, &kek_salt, policy.kek_kdf).await?;
    let wrapped_dek = keys.wrap_key(dek, kek.as_slice(), KeyRole::Dek, account_id)?;
    let wrapped_mac = keys.wrap_key(mac_key, kek.as_slice(), KeyRole::VaultMac, account_id)?;
    Ok(KeyMaterial {
        wrapped_dek: wrapped_dek.ciphertext,
        kek_salt,
        dek_nonce: wrapped_dek.nonce,
        wrapped_mac_key: Some(wrapped_mac.ciphertext),
        mac_key_nonce: Some(wrapped_mac.nonce),
        key_version: CURRENT_WRAP_VERSION,
        kdf: Some(policy.kek_kdf),
    })
}

/// Whether the stored key material should be re-sealed in the current format.
pub fn is_outdated(km: &KeyMaterial, policy: &AuthPolicy) -> bool {
    km.key_version < CURRENT_WRAP_VERSION
        || km.kdf != Some(policy.kek_kdf)
        || km.wrapped_mac_key.is_none()
}
