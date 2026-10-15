//! Tamper-evidence for the vault file.
//!
//! libSQL 0.9 only offers AES-256-CBC, which is *unauthenticated*: a modified
//! `vault.db` decrypts to garbage rather than being rejected. To detect at-rest
//! tampering we keep a keyed HMAC-SHA256 of the file, recorded on a clean close
//! and verified on open. This is tamper-*evidence*, not prevention, and only
//! covers the gap between clean sessions.
//!
//! The baseline and the "session in progress" flag live in the keystore (see
//! `integrity_store`), which is sealed by the per-device key: they cannot be
//! deleted or forged by someone with mere write access to the data directory.
//! Older versions kept them in `vault.db.hmac` / `vault.db.dirty` sidecars; those
//! are honoured exactly once, to migrate, and then removed (see [`assess`]).
//!
//! Forging a valid MAC requires the MAC key, which is wrapped by the
//! password-derived KEK — the same trust root as the DEK.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::domain::error::DomainError;
use crate::infrastructure::persistence::integrity_store::IntegrityRecord;

type HmacSha256 = Hmac<Sha256>;

/// Domain-separation tag of the current MAC format.
const MAC_V2_TAG: &[u8] = b"bp-vault-mac-v2";

/// Outcome of an at-rest integrity check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityVerdict {
    /// File matches the last clean-close baseline.
    Clean,
    /// No vault file and no baseline: a fresh vault, created on open.
    Fresh,
    /// The vault exists but no baseline was ever recorded anywhere: an install
    /// that predates tamper-evidence, or a crash before the very first close.
    NoBaseline,
    /// Mismatch, but the previous session did not close cleanly (a crash):
    /// expected, not suspicious.
    UncleanShutdown,
    /// Mismatch (or a vanished vault) after a clean previous shutdown: possible
    /// tampering.
    Mismatch,
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut os = path.as_os_str().to_owned();
    os.push(suffix);
    PathBuf::from(os)
}

fn legacy_sidecar_path(vault_path: &Path) -> PathBuf {
    with_suffix(vault_path, ".hmac")
}

fn legacy_dirty_path(vault_path: &Path) -> PathBuf {
    with_suffix(vault_path, ".dirty")
}

/// Stream a file into the MAC, returning how many bytes were fed.
fn feed_file(mac: &mut HmacSha256, path: &Path) -> Result<u64, DomainError> {
    let file = File::open(path).map_err(|e| DomainError::Storage(e.to_string()))?;
    let mut reader = BufReader::new(file);
    let mut buf = [0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        if n == 0 {
            return Ok(total);
        }
        mac.update(&buf[..n]);
        total += n as u64;
    }
}

fn new_mac(mac_key: &[u8]) -> Result<HmacSha256, DomainError> {
    HmacSha256::new_from_slice(mac_key).map_err(|e| DomainError::Crypto(e.to_string()))
}

/// Current MAC: HMAC-SHA256 over
/// `tag ‖ main ‖ len(main) ‖ has_wal ‖ wal ‖ len(wal)` (lengths as u64 LE).
///
/// Each variable-length segment is followed by its length, so the encoding is
/// injective: no two (main, wal) pairs hash the same input — unlike a plain
/// `main ‖ wal` concatenation. Files are streamed, never loaded whole.
pub fn compute_mac(vault_path: &Path, mac_key: &[u8]) -> Result<Vec<u8>, DomainError> {
    let mut mac = new_mac(mac_key)?;
    mac.update(MAC_V2_TAG);
    let main_len = feed_file(&mut mac, vault_path)?;
    mac.update(&main_len.to_le_bytes());
    let wal = with_suffix(vault_path, "-wal");
    if wal.exists() {
        mac.update(&[1]);
        let wal_len = feed_file(&mut mac, &wal)?;
        mac.update(&wal_len.to_le_bytes());
    } else {
        mac.update(&[0]);
    }
    Ok(mac.finalize().into_bytes().to_vec())
}

/// Legacy (v1) MAC: HMAC-SHA256 over `main ‖ wal`, as written by the sidecars of
/// earlier versions. Only used to migrate an existing sidecar baseline.
fn compute_legacy_mac(vault_path: &Path, mac_key: &[u8]) -> Result<Vec<u8>, DomainError> {
    let mut mac = new_mac(mac_key)?;
    feed_file(&mut mac, vault_path)?;
    let wal = with_suffix(vault_path, "-wal");
    if wal.exists() {
        feed_file(&mut mac, &wal)?;
    }
    Ok(mac.finalize().into_bytes().to_vec())
}

fn macs_equal(a: &[u8], b: &[u8]) -> bool {
    bool::from(a.ct_eq(b))
}

/// Verify the vault's at-rest state against the keystore record. Never opens
/// the DB.
///
/// Only the keystore record is trusted. The legacy sidecars are consulted when
/// the keystore holds no baseline yet — i.e. once, on the first open after the
/// upgrade — and are removed by [`remove_legacy_sidecars`] after that open.
pub fn assess(
    vault_path: &Path,
    mac_key: &[u8],
    record: &IntegrityRecord,
) -> Result<IntegrityVerdict, DomainError> {
    let exists = vault_path.exists();
    match (&record.baseline_mac, exists) {
        // A baseline exists but the vault is gone: data was removed behind our
        // back (a crash cannot delete the file). Refuse rather than silently
        // create an empty vault over it.
        (Some(_), false) => Ok(IntegrityVerdict::Mismatch),
        (None, false) => Ok(IntegrityVerdict::Fresh),
        (Some(expected), true) => {
            if macs_equal(expected, &compute_mac(vault_path, mac_key)?) {
                Ok(IntegrityVerdict::Clean)
            } else if record.session_open {
                Ok(IntegrityVerdict::UncleanShutdown)
            } else {
                Ok(IntegrityVerdict::Mismatch)
            }
        }
        (None, true) => assess_legacy(vault_path, mac_key, record),
    }
}

/// One-time path for vaults baselined by an earlier version (sidecar files).
fn assess_legacy(
    vault_path: &Path,
    mac_key: &[u8],
    record: &IntegrityRecord,
) -> Result<IntegrityVerdict, DomainError> {
    // A session already started under this version and never closed: the
    // sidecar (if any) is stale by construction — this is a crash.
    if record.session_open {
        return Ok(IntegrityVerdict::UncleanShutdown);
    }
    let Ok(expected) = std::fs::read(legacy_sidecar_path(vault_path)) else {
        return Ok(IntegrityVerdict::NoBaseline);
    };
    if macs_equal(&expected, &compute_legacy_mac(vault_path, mac_key)?) {
        Ok(IntegrityVerdict::Clean)
    } else if legacy_dirty_path(vault_path).exists() {
        Ok(IntegrityVerdict::UncleanShutdown)
    } else {
        Ok(IntegrityVerdict::Mismatch)
    }
}

/// Delete the legacy sidecars once their state has been taken over by the
/// keystore. Best-effort: they are ignored from then on anyway.
pub fn remove_legacy_sidecars(vault_path: &Path) {
    let _ = std::fs::remove_file(legacy_sidecar_path(vault_path));
    let _ = std::fs::remove_file(legacy_dirty_path(vault_path));
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [9u8; 32];

    fn temp_vault(name: &str, content: &[u8]) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("bp-integrity-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vault.db");
        std::fs::write(&path, content).unwrap();
        (dir, path)
    }

    fn record(mac: Option<Vec<u8>>, session_open: bool) -> IntegrityRecord {
        IntegrityRecord {
            baseline_mac: mac,
            session_open,
        }
    }

    #[test]
    fn matching_baseline_is_clean() {
        let (dir, path) = temp_vault("clean", b"hello");
        let mac = compute_mac(&path, &KEY).unwrap();
        let verdict = assess(&path, &KEY, &record(Some(mac), false)).unwrap();
        assert_eq!(verdict, IntegrityVerdict::Clean);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn modification_after_clean_close_is_a_mismatch_whatever_the_sidecars() {
        let (dir, path) = temp_vault("tamper", b"hello");
        let mac = compute_mac(&path, &KEY).unwrap();
        std::fs::write(&path, b"hellO").unwrap();
        // An attacker planting a legacy dirty marker or sidecar changes nothing.
        std::fs::write(legacy_dirty_path(&path), b"open").unwrap();
        std::fs::write(legacy_sidecar_path(&path), b"junk").unwrap();
        let verdict = assess(&path, &KEY, &record(Some(mac), false)).unwrap();
        assert_eq!(verdict, IntegrityVerdict::Mismatch);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn modification_during_open_session_is_an_unclean_shutdown() {
        let (dir, path) = temp_vault("crash", b"hello");
        let mac = compute_mac(&path, &KEY).unwrap();
        std::fs::write(&path, b"hellO").unwrap();
        let verdict = assess(&path, &KEY, &record(Some(mac), true)).unwrap();
        assert_eq!(verdict, IntegrityVerdict::UncleanShutdown);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn vanished_vault_with_a_baseline_is_a_mismatch() {
        let (dir, path) = temp_vault("gone", b"hello");
        let mac = compute_mac(&path, &KEY).unwrap();
        std::fs::remove_file(&path).unwrap();
        let verdict = assess(&path, &KEY, &record(Some(mac), false)).unwrap();
        assert_eq!(verdict, IntegrityVerdict::Mismatch);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn legacy_sidecar_is_honoured_once() {
        let (dir, path) = temp_vault("legacy", b"hello");
        let legacy = compute_legacy_mac(&path, &KEY).unwrap();
        std::fs::write(legacy_sidecar_path(&path), &legacy).unwrap();
        let empty = record(None, false);
        assert_eq!(
            assess(&path, &KEY, &empty).unwrap(),
            IntegrityVerdict::Clean
        );
        std::fs::write(&path, b"hellO").unwrap();
        assert_eq!(
            assess(&path, &KEY, &empty).unwrap(),
            IntegrityVerdict::Mismatch
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn length_framing_distinguishes_main_wal_splits() {
        let (dir, path) = temp_vault("split", b"abcd");
        let a = compute_mac(&path, &KEY).unwrap();
        std::fs::write(&path, b"ab").unwrap();
        std::fs::write(with_suffix(&path, "-wal"), b"cd").unwrap();
        let b = compute_mac(&path, &KEY).unwrap();
        assert_ne!(a, b);
        let _ = std::fs::remove_dir_all(dir);
    }
}
