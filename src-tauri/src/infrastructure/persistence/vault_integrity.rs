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
//!
//! # Why the sidecar carries its own header MAC
//!
//! An earlier layout kept the crash marker in a *separate, unauthenticated*
//! file (`vault.db.dirty`) and treated a missing sidecar as "no baseline yet".
//! Both were fail-open shortcuts an attacker could take: `touch vault.db.dirty`
//! downgraded a genuine mismatch to "expected after a crash", and deleting the
//! sidecar skipped verification entirely. Whoever can modify the vault can also
//! create or delete a file next to it, so neither signal could be trusted.
//!
//! The state now lives *inside* the sidecar, under a MAC:
//!
//! ```text
//! version(1) │ flag(1) │ counter(8 LE) │ content_mac(32) │ header_mac(32)
//! ```
//!
//! - `header_mac` covers version + flag + counter + content_mac, so the crash
//!   flag cannot be forged without the MAC key.
//! - `content_mac` covers the session counter, the length of the main file, and
//!   then the main file and its `-wal`. Including the length removes the
//!   `main ‖ wal` concatenation ambiguity.
//! - `counter` is a monotonic session number anchored in the keystore (which is
//!   itself sealed by the OS-keychain device key). Replaying an older
//!   vault + sidecar pair is therefore detected: the counter no longer matches.

use std::path::{Path, PathBuf};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::domain::error::DomainError;

type HmacSha256 = Hmac<Sha256>;

/// Sidecar layout version. Bump when the byte layout changes.
const SIDECAR_VERSION: u8 = 2;
/// version + flag + counter + content_mac + header_mac.
const SIDECAR_LEN: usize = 1 + 1 + 8 + 32 + 32;

/// Domain separators, so a digest computed for one purpose can never be
/// accepted for the other.
const HEADER_DOMAIN: &[u8] = b"basic-presence/vault-sidecar-header/v2";
const CONTENT_DOMAIN: &[u8] = b"basic-presence/vault-sidecar-content/v2";

/// Whether the previous session ended cleanly. Stored inside the authenticated
/// sidecar, never as a bare file on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// A session is in progress (or the process died during one).
    Open,
    /// The vault was closed cleanly and the baseline reflects the final file.
    Closed,
}

impl SessionState {
    fn as_byte(self) -> u8 {
        match self {
            SessionState::Open => 0,
            SessionState::Closed => 1,
        }
    }

    fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(SessionState::Open),
            1 => Some(SessionState::Closed),
            _ => None,
        }
    }
}

/// Outcome of an at-rest integrity check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityVerdict {
    /// File matches the last clean-close baseline.
    Clean,
    /// No baseline yet, and none is expected (fresh vault, or first run after
    /// the feature shipped). Only ever returned when the keystore counter is 0.
    NoBaseline,
    /// The previous session did not close cleanly (a crash): the flag saying so
    /// is itself authenticated, so this is not a forgeable shortcut.
    UncleanShutdown,
    /// Mismatch, missing baseline that should exist, forged sidecar, or a
    /// counter from an older session: treat as tampering.
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

/// Legacy (v1) marker file. Only ever removed, never trusted.
fn legacy_dirty_path(vault_path: &Path) -> PathBuf {
    with_suffix(vault_path, ".dirty")
}

fn mac(key: &[u8], domain: &[u8]) -> Result<HmacSha256, DomainError> {
    let mut m = HmacSha256::new_from_slice(key).map_err(|e| DomainError::Crypto(e.to_string()))?;
    m.update(domain);
    Ok(m)
}

/// HMAC over the session counter, the main file (length-prefixed) and its
/// `-wal`, if a write has not yet been checkpointed.
fn compute_content_mac(
    vault_path: &Path,
    mac_key: &[u8],
    counter: u64,
) -> Result<Vec<u8>, DomainError> {
    let mut m = mac(mac_key, CONTENT_DOMAIN)?;
    m.update(&counter.to_le_bytes());
    let main = std::fs::read(vault_path).map_err(|e| DomainError::Storage(e.to_string()))?;
    // Length-prefixed so `main ‖ wal` cannot be re-split ambiguously.
    m.update(&(main.len() as u64).to_le_bytes());
    m.update(&main);
    if let Ok(wal) = std::fs::read(with_suffix(vault_path, "-wal")) {
        m.update(&wal);
    }
    Ok(m.finalize().into_bytes().to_vec())
}

fn compute_header_mac(
    mac_key: &[u8],
    state: SessionState,
    counter: u64,
    content_mac: &[u8],
) -> Result<Vec<u8>, DomainError> {
    let mut m = mac(mac_key, HEADER_DOMAIN)?;
    m.update(&[SIDECAR_VERSION, state.as_byte()]);
    m.update(&counter.to_le_bytes());
    m.update(content_mac);
    Ok(m.finalize().into_bytes().to_vec())
}

struct Sidecar {
    state: SessionState,
    counter: u64,
    content_mac: Vec<u8>,
    header_mac: Vec<u8>,
}

fn parse(bytes: &[u8]) -> Option<Sidecar> {
    if bytes.len() != SIDECAR_LEN || bytes[0] != SIDECAR_VERSION {
        return None;
    }
    let state = SessionState::from_byte(bytes[1])?;
    let mut counter_bytes = [0u8; 8];
    counter_bytes.copy_from_slice(&bytes[2..10]);
    Some(Sidecar {
        state,
        counter: u64::from_le_bytes(counter_bytes),
        content_mac: bytes[10..42].to_vec(),
        header_mac: bytes[42..74].to_vec(),
    })
}

/// Verify the vault's at-rest state against its baseline. Never opens the DB.
///
/// `expected_counter` is the session counter held in the sealed keystore. A
/// value of 0 means this vault has never been baselined by this build, which is
/// the only situation where a missing or legacy sidecar is acceptable.
pub fn verify(
    vault_path: &Path,
    mac_key: &[u8],
    expected_counter: u64,
) -> Result<IntegrityVerdict, DomainError> {
    if !vault_path.exists() {
        return Ok(IntegrityVerdict::NoBaseline); // fresh vault, created on open
    }

    let Ok(raw) = std::fs::read(sidecar_path(vault_path)) else {
        // A vault that has already been baselined must still have its sidecar.
        // Its absence is now a finding, not a free pass.
        return Ok(if expected_counter == 0 {
            IntegrityVerdict::NoBaseline
        } else {
            IntegrityVerdict::Mismatch
        });
    };

    let Some(sidecar) = parse(&raw) else {
        // Unparseable or v1 layout. Accepted once, only while migrating from the
        // old format (counter still 0); afterwards it is a forged/corrupt file.
        return Ok(if expected_counter == 0 {
            IntegrityVerdict::NoBaseline
        } else {
            IntegrityVerdict::Mismatch
        });
    };

    // Authenticate the header first: everything below trusts `state` and
    // `counter`, so they must be proven to come from the key holder.
    let expected_header = compute_header_mac(
        mac_key,
        sidecar.state,
        sidecar.counter,
        &sidecar.content_mac,
    )?;
    if !bool::from(expected_header.ct_eq(&sidecar.header_mac)) {
        return Ok(IntegrityVerdict::Mismatch);
    }

    // A sidecar from an earlier session means the pair (vault, sidecar) was
    // rolled back wholesale.
    if sidecar.counter != expected_counter {
        return Ok(IntegrityVerdict::Mismatch);
    }

    // The crash flag is authenticated, so honouring it is safe.
    if sidecar.state == SessionState::Open {
        return Ok(IntegrityVerdict::UncleanShutdown);
    }

    let actual = compute_content_mac(vault_path, mac_key, sidecar.counter)?;
    if bool::from(actual.ct_eq(&sidecar.content_mac)) {
        Ok(IntegrityVerdict::Clean)
    } else {
        Ok(IntegrityVerdict::Mismatch)
    }
}

/// Write the sidecar for the current session.
///
/// Called with [`SessionState::Open`] *before the first byte is written* to the
/// vault, and with [`SessionState::Closed`] once the database is flushed. Both
/// record the file state at the time of the call.
pub fn write_marker(
    vault_path: &Path,
    mac_key: &[u8],
    counter: u64,
    state: SessionState,
) -> Result<(), DomainError> {
    // A vault that does not exist yet (first open) has no content to cover; the
    // Closed marker written at the end of the session will carry the real one.
    let content_mac = if vault_path.exists() {
        compute_content_mac(vault_path, mac_key, counter)?
    } else {
        mac(mac_key, CONTENT_DOMAIN)?
            .finalize()
            .into_bytes()
            .to_vec()
    };
    let header_mac = compute_header_mac(mac_key, state, counter, &content_mac)?;

    let mut out = Vec::with_capacity(SIDECAR_LEN);
    out.push(SIDECAR_VERSION);
    out.push(state.as_byte());
    out.extend_from_slice(&counter.to_le_bytes());
    out.extend_from_slice(&content_mac);
    out.extend_from_slice(&header_mac);

    std::fs::write(sidecar_path(vault_path), &out)
        .map_err(|e| DomainError::Storage(e.to_string()))?;
    // Clean up the v1 marker if this install is coming from the old format.
    let _ = std::fs::remove_file(legacy_dirty_path(vault_path));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [9u8; 32];

    struct Vault {
        dir: PathBuf,
        path: PathBuf,
    }

    impl Vault {
        fn new(name: &str) -> Self {
            let mut dir = std::env::temp_dir();
            dir.push(format!("bp-integrity-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("temp dir");
            let path = dir.join("vault.db");
            std::fs::write(&path, b"original contents").expect("write vault");
            Self { dir, path }
        }

        fn sidecar(&self) -> PathBuf {
            sidecar_path(&self.path)
        }
    }

    impl Drop for Vault {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn a_clean_close_verifies() {
        let v = Vault::new("clean");
        write_marker(&v.path, &KEY, 1, SessionState::Closed).unwrap();
        assert_eq!(verify(&v.path, &KEY, 1).unwrap(), IntegrityVerdict::Clean);
    }

    #[test]
    fn modified_contents_are_detected() {
        let v = Vault::new("modified");
        write_marker(&v.path, &KEY, 1, SessionState::Closed).unwrap();
        std::fs::write(&v.path, b"tampered contents!").unwrap();
        assert_eq!(
            verify(&v.path, &KEY, 1).unwrap(),
            IntegrityVerdict::Mismatch
        );
    }

    /// The v1 regression: creating a bare marker file next to the vault used to
    /// downgrade a mismatch to "crash, nothing to see here".
    #[test]
    fn a_bare_dirty_file_no_longer_excuses_a_mismatch() {
        let v = Vault::new("bare-dirty");
        write_marker(&v.path, &KEY, 1, SessionState::Closed).unwrap();
        std::fs::write(&v.path, b"tampered contents!").unwrap();
        std::fs::write(legacy_dirty_path(&v.path), b"open").unwrap();

        assert_eq!(
            verify(&v.path, &KEY, 1).unwrap(),
            IntegrityVerdict::Mismatch,
            "an unauthenticated marker must not suppress the finding"
        );
    }

    /// The other v1 regression: deleting the sidecar used to skip verification.
    #[test]
    fn deleting_the_sidecar_is_a_mismatch_once_baselined() {
        let v = Vault::new("deleted-sidecar");
        write_marker(&v.path, &KEY, 1, SessionState::Closed).unwrap();
        std::fs::remove_file(v.sidecar()).unwrap();

        assert_eq!(
            verify(&v.path, &KEY, 1).unwrap(),
            IntegrityVerdict::Mismatch,
            "a vault that has been baselined must still carry its sidecar"
        );
    }

    #[test]
    fn a_missing_sidecar_is_accepted_only_before_the_first_baseline() {
        let v = Vault::new("legacy");
        assert_eq!(
            verify(&v.path, &KEY, 0).unwrap(),
            IntegrityVerdict::NoBaseline
        );
    }

    #[test]
    fn a_legacy_v1_sidecar_is_accepted_once_then_replaced() {
        let v = Vault::new("legacy-v1");
        // v1 sidecar was a bare 32-byte digest.
        std::fs::write(v.sidecar(), [0u8; 32]).unwrap();
        assert_eq!(
            verify(&v.path, &KEY, 0).unwrap(),
            IntegrityVerdict::NoBaseline
        );
        // …but not once the counter has moved on.
        assert_eq!(
            verify(&v.path, &KEY, 1).unwrap(),
            IntegrityVerdict::Mismatch
        );
    }

    #[test]
    fn an_authenticated_open_marker_reports_an_unclean_shutdown() {
        let v = Vault::new("crash");
        write_marker(&v.path, &KEY, 3, SessionState::Open).unwrap();
        // A crash mid-write leaves the file different from the marker.
        std::fs::write(&v.path, b"half-written state").unwrap();
        assert_eq!(
            verify(&v.path, &KEY, 3).unwrap(),
            IntegrityVerdict::UncleanShutdown
        );
    }

    #[test]
    fn rolling_back_to_an_older_session_is_detected() {
        let v = Vault::new("rollback");
        // Session 1 closes cleanly; the attacker keeps a copy of both files.
        write_marker(&v.path, &KEY, 1, SessionState::Closed).unwrap();
        let old_vault = std::fs::read(&v.path).unwrap();
        let old_sidecar = std::fs::read(v.sidecar()).unwrap();

        // Sessions carry on; the keystore counter is now 5.
        std::fs::write(&v.path, b"newer contents with more days").unwrap();
        write_marker(&v.path, &KEY, 5, SessionState::Closed).unwrap();

        // The attacker restores the whole consistent pair from session 1.
        std::fs::write(&v.path, &old_vault).unwrap();
        std::fs::write(v.sidecar(), &old_sidecar).unwrap();

        assert_eq!(
            verify(&v.path, &KEY, 5).unwrap(),
            IntegrityVerdict::Mismatch,
            "a self-consistent but stale pair must be rejected"
        );
    }

    #[test]
    fn a_forged_sidecar_without_the_key_is_rejected() {
        let v = Vault::new("forged");
        write_marker(&v.path, &KEY, 2, SessionState::Closed).unwrap();
        // Attacker flips the state byte to Open, hoping for UncleanShutdown.
        let mut raw = std::fs::read(v.sidecar()).unwrap();
        raw[1] = SessionState::Open.as_byte();
        std::fs::write(v.sidecar(), &raw).unwrap();

        assert_eq!(
            verify(&v.path, &KEY, 2).unwrap(),
            IntegrityVerdict::Mismatch
        );
    }

    #[test]
    fn another_key_cannot_validate_the_sidecar() {
        let v = Vault::new("wrong-key");
        write_marker(&v.path, &KEY, 1, SessionState::Closed).unwrap();
        assert_eq!(
            verify(&v.path, &[1u8; 32], 1).unwrap(),
            IntegrityVerdict::Mismatch
        );
    }

    /// Length-prefixing the main file removes the `main ‖ wal` ambiguity: two
    /// different splits of the same bytes must not produce the same digest.
    #[test]
    fn the_main_wal_split_is_unambiguous() {
        let a = Vault::new("split-a");
        std::fs::write(&a.path, b"abcdef").unwrap();
        std::fs::write(with_suffix(&a.path, "-wal"), b"ghi").unwrap();
        let mac_a = compute_content_mac(&a.path, &KEY, 1).unwrap();

        let b = Vault::new("split-b");
        std::fs::write(&b.path, b"abc").unwrap();
        std::fs::write(with_suffix(&b.path, "-wal"), b"defghi").unwrap();
        let mac_b = compute_content_mac(&b.path, &KEY, 1).unwrap();

        assert_ne!(mac_a, mac_b);
    }
}
