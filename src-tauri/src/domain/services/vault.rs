use async_trait::async_trait;

use crate::domain::error::DomainError;

/// Port controlling the encrypted vault database lifecycle.
///
/// The vault is opened (decrypted) at login with the unwrapped DEK and locked
/// (connection dropped, key forgotten) at logout/expiry.
#[async_trait]
pub trait VaultManager: Send + Sync {
    /// Open/decrypt the vault with the given 32-byte DEK and run its migrations.
    async fn open(&self, dek: &[u8]) -> Result<(), DomainError>;

    /// Lock the vault (drop the connection / forget the key).
    fn close(&self);
}
