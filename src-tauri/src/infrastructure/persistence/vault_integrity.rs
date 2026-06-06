//! Tamper-evidence for the vault file.
//!
//! libSQL 0.9 only offers AES-256-CBC, which is *unauthenticated*: a modified
//! `vault.db` decrypts to garbage rather than being rejected. To detect at-rest
//! tampering we keep a keyed HMAC-SHA256 of the file in a sidecar, written on a
//! clean close and verified on open. This is tamper-*evidence*, not prevention,
//! and only covers the gap between clean sessions.
//!
//! Forging a valid sidecar requires the MAC key, which is wrapped by the
//! password-derived KEK — the same trust root as the DEK.

use std::path::{Path, PathBuf};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::domain::error::DomainError;

type HmacSha256 = Hmac<Sha256>;

/// Outcome of an at-rest integrity check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityVerdict {
    /// File matches the last clean-close baseline.
    Clean,
    /// No baseline yet (fresh vault, or first run after the feature shipped).
    NoBaseline,
    /// Mismatch, but the previous session did not close cleanly (a crash):
    /// expected, not suspicious.
    UncleanShutdown,
    /// Mismatch after a clean previous shutdown: possible tampering.
    Mismatch,
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut os = path.as_os_str().to_owned();
    os.push(suffix);
    PathBuf::from(os)
}

fn sidecar_path(vault_path: &Path) -> PathBuf {
    with_suffix(vault_path, ".hmac")
}

fn dirty_path(vault_path: &Path) -> PathBuf {
    with_suffix(vault_path, ".dirty")
}

/// HMAC-SHA256 over the vault file (plus its `-wal`, if a write has not yet been
/// checkpointed) so the digest covers the complete on-disk state.
fn compute_mac(vault_path: &Path, mac_key: &[u8]) -> Result<Vec<u8>, DomainError> {
    let mut mac =
        HmacSha256::new_from_slice(mac_key).map_err(|e| DomainError::Crypto(e.to_string()))?;
    let main = std::fs::read(vault_path).map_err(|e| DomainError::Storage(e.to_string()))?;
    mac.update(&main);
    if let Ok(wal) = std::fs::read(with_suffix(vault_path, "-wal")) {
        mac.update(&wal);
    }
    Ok(mac.finalize().into_bytes().to_vec())
}

/// Verify the vault's at-rest state against its baseline. Never opens the DB.
pub fn verify(vault_path: &Path, mac_key: &[u8]) -> Result<IntegrityVerdict, DomainError> {
    if !vault_path.exists() {
        return Ok(IntegrityVerdict::NoBaseline); // fresh vault, created on open
    }
    let Ok(expected) = std::fs::read(sidecar_path(vault_path)) else {
        return Ok(IntegrityVerdict::NoBaseline); // first run after the feature shipped
    };
    let actual = compute_mac(vault_path, mac_key)?;
    if bool::from(expected.ct_eq(&actual)) {
        Ok(IntegrityVerdict::Clean)
    } else if dirty_path(vault_path).exists() {
        Ok(IntegrityVerdict::UncleanShutdown)
    } else {
        Ok(IntegrityVerdict::Mismatch)
    }
}

/// Mark a session as in progress: if the process dies before a clean close, the
/// next `verify` finds this marker and treats a mismatch as an unclean shutdown
/// rather than tampering.
pub fn mark_dirty(vault_path: &Path) -> Result<(), DomainError> {
    std::fs::write(dirty_path(vault_path), b"open").map_err(|e| DomainError::Storage(e.to_string()))
}

/// Record the current file state as the trusted baseline (called on a clean
/// close, once the DB is flushed) and clear the dirty marker.
pub fn write_baseline(vault_path: &Path, mac_key: &[u8]) -> Result<(), DomainError> {
    let mac = compute_mac(vault_path, mac_key)?;
    std::fs::write(sidecar_path(vault_path), &mac)
        .map_err(|e| DomainError::Storage(e.to_string()))?;
    let _ = std::fs::remove_file(dirty_path(vault_path));
    Ok(())
}
