use async_trait::async_trait;
use zeroize::Zeroizing;

use crate::domain::error::DomainError;
use crate::domain::policy::KdfParams;

/// Version of the key-wrapping format.
///
/// - **1**: XChaCha20-Poly1305 without associated data (legacy).
/// - **2**: XChaCha20-Poly1305 with associated data binding the wrapped key to
///   its role and owner account (see [`KeyBinding`]).
pub const CURRENT_WRAP_VERSION: i64 = 2;

/// A wrapped (encrypted) key: AEAD ciphertext + nonce.
#[derive(Debug, Clone)]
pub struct WrappedKey {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// What a wrapped key is for. Part of the AEAD associated data, so a wrapped
/// DEK can never be accepted where the MAC key is expected (or vice versa).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRole {
    /// Data Encryption Key of the vault.
    Dek,
    /// Key of the vault tamper-evidence MAC.
    VaultMac,
}

/// Context a key is wrapped under: format version, role and owner account.
#[derive(Debug, Clone, Copy)]
pub struct KeyBinding<'a> {
    pub version: i64,
    pub role: KeyRole,
    pub account_id: &'a str,
}

/// Port for the envelope-encryption key operations.
///
/// A random Data Encryption Key (DEK) encrypts the vault; the DEK is wrapped by
/// a Key Encryption Key (KEK) derived from the user's password.
#[async_trait]
pub trait KeyService: Send + Sync {
    /// Derive a 32-byte KEK from a password + salt (Argon2id) with the given
    /// cost parameters. Expensive: runs off the async executor.
    async fn derive_kek(
        &self,
        password: &[u8],
        salt: &[u8],
        params: KdfParams,
    ) -> Result<Zeroizing<[u8; 32]>, DomainError>;

    /// Generate a fresh random 32-byte key (DEK or MAC key).
    fn generate_key(&self) -> Result<Zeroizing<[u8; 32]>, DomainError>;

    /// Generate a fresh random salt (for KEK derivation).
    fn generate_salt(&self) -> Result<Vec<u8>, DomainError>;

    /// Encrypt (wrap) a key with the KEK using an authenticated AEAD, in the
    /// current format.
    fn wrap_key(
        &self,
        key: &[u8],
        kek: &[u8],
        role: KeyRole,
        account_id: &str,
    ) -> Result<WrappedKey, DomainError>;

    /// Decrypt (unwrap) a key with the KEK, in the format it was wrapped with.
    fn unwrap_key(
        &self,
        wrapped: &WrappedKey,
        kek: &[u8],
        binding: KeyBinding<'_>,
    ) -> Result<Zeroizing<[u8; 32]>, DomainError>;
}
