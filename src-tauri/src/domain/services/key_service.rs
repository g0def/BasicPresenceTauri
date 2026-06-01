use zeroize::Zeroizing;

use crate::domain::error::DomainError;

/// A wrapped (encrypted) key: AEAD ciphertext + nonce.
#[derive(Debug, Clone)]
pub struct WrappedKey {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Port for the envelope-encryption key operations.
///
/// A random Data Encryption Key (DEK) encrypts the vault; the DEK is wrapped by
/// a Key Encryption Key (KEK) derived from the user's password.
pub trait KeyService: Send + Sync {
    /// Derive a 32-byte KEK from a password + salt (Argon2id).
    fn derive_kek(&self, password: &[u8], salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, DomainError>;

    /// Generate a fresh random 32-byte Data Encryption Key.
    fn generate_dek(&self) -> Result<Zeroizing<[u8; 32]>, DomainError>;

    /// Generate a fresh random salt (for KEK derivation).
    fn generate_salt(&self) -> Result<Vec<u8>, DomainError>;

    /// Encrypt (wrap) the DEK with the KEK using an authenticated AEAD.
    fn wrap_dek(&self, dek: &[u8], kek: &[u8]) -> Result<WrappedKey, DomainError>;

    /// Decrypt (unwrap) the DEK with the KEK.
    fn unwrap_dek(
        &self,
        wrapped: &WrappedKey,
        kek: &[u8],
    ) -> Result<Zeroizing<[u8; 32]>, DomainError>;
}
