use zeroize::Zeroizing;

use crate::domain::error::DomainError;

/// Argon2id cost parameters used for one KEK derivation.
///
/// Stored with the account rather than read from the running configuration:
/// the same password must derive the same KEK for the lifetime of a vault, so
/// these belong to the key material, not to the build. Raising the cost for new
/// accounts is then safe — existing ones keep deriving with the values they were
/// created under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfParams {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

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
    /// Derive a 32-byte KEK from a password + salt (Argon2id), using the cost
    /// parameters the account was created with.
    fn derive_kek(
        &self,
        password: &[u8],
        salt: &[u8],
        params: KdfParams,
    ) -> Result<Zeroizing<[u8; 32]>, DomainError>;

    /// Cost parameters this build uses for *new* accounts.
    fn default_params(&self) -> KdfParams;

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
