use keyring::{Entry, Error as KeyringError};
use zeroize::Zeroizing;

use crate::domain::error::DomainError;

/// Keychain coordinates for the per-device keystore-encryption key.
const KEYRING_SERVICE: &str = "com.godef.basic-presence";
const KEYRING_ACCOUNT: &str = "keystore-device-key";

/// Fetch the per-device key used to encrypt `keystore.db` at rest, generating
/// and persisting a fresh random 32-byte key in the OS keychain on first run.
///
/// This key is **per device, not per account**, so it is resolved once at
/// startup, independently of whether an owner account exists yet. It seals the
/// keystore behind the OS keychain (GNOME Keyring/KWallet, macOS Keychain,
/// Windows Credential Manager): an attacker who only steals the `keystore.db`
/// file can no longer brute-force the password offline without also extracting
/// this key from the keychain.
///
/// On Linux a Secret Service provider must be running; if it is unavailable the
/// call returns [`DomainError::Keychain`] (surfaced as a startup error rather
/// than silently downgrading to a plaintext keystore).
pub fn resolve_device_key() -> Result<Zeroizing<[u8; 32]>, DomainError> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|e| DomainError::Keychain(e.to_string()))?;

    match entry.get_secret() {
        Ok(bytes) => {
            if bytes.len() != 32 {
                return Err(DomainError::Keychain(
                    "stored device key has unexpected length".into(),
                ));
            }
            let mut key = Zeroizing::new([0u8; 32]);
            key.as_mut_slice().copy_from_slice(&bytes);
            Ok(key)
        }
        Err(KeyringError::NoEntry) => {
            // First run on this device: mint and persist a new key.
            let mut key = Zeroizing::new([0u8; 32]);
            getrandom::fill(key.as_mut_slice()).map_err(|e| DomainError::Crypto(e.to_string()))?;
            entry
                .set_secret(key.as_slice())
                .map_err(|e| DomainError::Keychain(e.to_string()))?;
            Ok(key)
        }
        Err(e) => Err(DomainError::Keychain(e.to_string())),
    }
}
