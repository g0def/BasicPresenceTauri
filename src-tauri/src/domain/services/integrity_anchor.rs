use async_trait::async_trait;

use crate::domain::error::DomainError;

/// Anchor for the vault's at-rest integrity state, held somewhere an attacker
/// with only the data directory cannot rewrite (in practice: the keystore,
/// sealed by the OS-keychain device key).
///
/// The vault sidecar carries a session counter; this port is what the counter is
/// compared against. Without such an anchor, an attacker could restore an older
/// but internally consistent `vault.db` + sidecar pair and it would verify.
#[async_trait]
pub trait IntegrityAnchor: Send + Sync {
    /// Counter of the session the current on-disk baseline belongs to.
    /// `0` means the vault has never been baselined by this build.
    async fn counter(&self) -> Result<u64, DomainError>;

    /// Start a new session: increment and return the new counter.
    async fn bump(&self) -> Result<u64, DomainError>;
}
