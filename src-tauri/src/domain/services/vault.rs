use async_trait::async_trait;

use crate::domain::error::DomainError;

/// Port controlling the encrypted vault database lifecycle.
///
/// The vault is opened (decrypted) at login with the unwrapped DEK and locked
/// (connection dropped, key forgotten) at logout/expiry.
#[async_trait]
pub trait VaultManager: Send + Sync {
    /// Open/decrypt the vault with the given 32-byte DEK and run its migrations.
    /// The `mac_key` (also 32 bytes) is used to verify the at-rest integrity of
    /// the file on open and to re-baseline it on close.
    async fn open(&self, dek: &[u8], mac_key: &[u8]) -> Result<(), DomainError>;

    /// Lock the vault (drop the connection / forget the key) and, if it was
    /// open, refresh the at-rest integrity baseline.
    fn close(&self);
}
