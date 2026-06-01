use super::user::User;

/// Cryptographic material that protects the encrypted vault, stored alongside
/// the auth credentials in the (unencrypted) keystore.
///
/// Every field here is already cryptographically protected (the DEK is wrapped,
/// the salt is public by design), so storing it in clear is safe.
#[derive(Debug, Clone)]
pub struct KeyMaterial {
    /// DEK encrypted (wrapped) with the password-derived KEK (AEAD ciphertext).
    pub wrapped_dek: Vec<u8>,
    /// Salt used to derive the KEK from the password (Argon2id).
    pub kek_salt: Vec<u8>,
    /// AEAD nonce used when wrapping the DEK (24 bytes, XChaCha20-Poly1305).
    pub dek_nonce: Vec<u8>,
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
