use super::user::User;
use crate::domain::policy::KdfParams;

/// Cryptographic material that protects the encrypted vault, stored alongside
/// the auth credentials in the keystore.
///
/// Every field here is already cryptographically protected (the keys are
/// wrapped, the salt is public by design), so storing it is safe even before
/// the keystore's own at-rest sealing.
#[derive(Debug, Clone)]
pub struct KeyMaterial {
    /// DEK encrypted (wrapped) with the password-derived KEK (AEAD ciphertext).
    pub wrapped_dek: Vec<u8>,
    /// Salt used to derive the KEK from the password (Argon2id).
    pub kek_salt: Vec<u8>,
    /// AEAD nonce used when wrapping the DEK (24 bytes, XChaCha20-Poly1305).
    pub dek_nonce: Vec<u8>,
    /// MAC key (for vault tamper-evidence) wrapped with the same KEK. `None`
    /// for accounts created before the feature; backfilled on next login.
    pub wrapped_mac_key: Option<Vec<u8>>,
    /// AEAD nonce used when wrapping the MAC key.
    pub mac_key_nonce: Option<Vec<u8>>,
    /// Wrap format version (see `key_service::CURRENT_WRAP_VERSION`).
    pub key_version: i64,
    /// Argon2id parameters the KEK was derived with; `None` for legacy
    /// accounts (derived with the policy's `legacy_kek_kdf`).
    pub kdf: Option<KdfParams>,
}

/// The single device-owner account: auth credentials + key material + lockout state.
#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub username: String,
    /// Argon2id PHC string. Never leaves the backend.
    pub password_hash: String,
    pub key_material: KeyMaterial,
    pub failed_attempts: i64,
    pub locked_until: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Account {
    pub fn to_user(&self) -> User {
        User {
            id: self.id.clone(),
            username: self.username.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
