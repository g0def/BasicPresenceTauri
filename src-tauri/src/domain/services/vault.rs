use async_trait::async_trait;

use crate::domain::error::DomainError;

/// What the at-rest integrity check found when the vault was opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultOpenOutcome {
    /// The vault matched its last clean-close baseline (or is brand new).
    Clean,
    /// The previous session did not close cleanly (crash, power loss, kill):
    /// the file legitimately differs from the baseline. Worth telling the user.
    RecoveredFromUncleanShutdown,
    /// The file differs from the baseline after a clean shutdown, and the
    /// configured policy tolerates it (never the case in production).
    TamperingTolerated,
}

/// Port controlling the encrypted vault database lifecycle.
///
/// The vault is opened (decrypted) at login with the unwrapped DEK and locked
/// (connection dropped, key forgotten) at logout/expiry.
#[async_trait]
pub trait VaultManager: Send + Sync {
    /// Open/decrypt the vault with the given 32-byte DEK and run its migrations.
    /// The `mac_key` (also 32 bytes) is used to verify the at-rest integrity of
    /// the file on open and to re-baseline it on close.
    async fn open(&self, dek: &[u8], mac_key: &[u8]) -> Result<VaultOpenOutcome, DomainError>;

    /// Lock the vault (drop the connection / forget the key) and, if it was
    /// open, refresh the at-rest integrity baseline. Waits for in-flight writes
    /// to finish first, so the baseline covers the final file state.
    async fn close(&self);
}
