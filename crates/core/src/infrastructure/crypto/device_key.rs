use keyring_core::api::CredentialStoreApi as _;
use keyring_core::{Entry, Error as KeyringError};
use zeroize::Zeroizing;

use crate::domain::error::DomainError;

/// Keychain coordinates for the per-device keystore-encryption key.
const KEYRING_SERVICE: &str = "com.godef.basic-presence";
const KEYRING_ACCOUNT: &str = "keystore-device-key";

/// The keychain entry, in the OS's native store. These stores look items up by
/// the same coordinates keyring 3 wrote them with (macOS generic password
/// service/account, Windows target `{account}.{service}`, Secret Service
/// `service`/`username` attributes), so existing device keys are found.
fn keychain_entry() -> Result<Entry, KeyringError> {
    #[cfg(target_os = "macos")]
    let store = apple_native_keyring_store::keychain::Store::new()?;
    #[cfg(target_os = "windows")]
    let store = windows_native_keyring_store::Store::new()?;
    #[cfg(all(unix, not(target_os = "macos")))]
    let store = zbus_secret_service_keyring_store::Store::new()?;
    store.build(KEYRING_SERVICE, KEYRING_ACCOUNT, None)
}

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
///
/// `allow_create` must be `false` once a sealed keystore exists: a missing entry
/// then means the key was lost (keychain reset, profile restored elsewhere…).
/// Minting a fresh key would not help — the keystore stays undecryptable — and
/// would overwrite any chance of restoring the original entry, so this returns
/// [`DomainError::KeystoreUnrecoverable`] instead.
pub fn resolve_device_key(allow_create: bool) -> Result<Zeroizing<[u8; 32]>, DomainError> {
    let entry = keychain_entry().map_err(|e| DomainError::Keychain(e.to_string()))?;

    match entry.get_secret() {
        Ok(bytes) => {
            let bytes = Zeroizing::new(bytes);
            if bytes.len() != 32 {
                return Err(DomainError::Keychain(
                    "stored device key has unexpected length".into(),
                ));
            }
            let mut key = Zeroizing::new([0u8; 32]);
            key.as_mut_slice().copy_from_slice(&bytes);
            Ok(key)
        }
        Err(KeyringError::NoEntry) if !allow_create => Err(DomainError::KeystoreUnrecoverable),
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

#[cfg(test)]
mod tests {
    /// Reads this machine's real device key — written by an earlier build, e.g.
    /// with keyring 3 — without ever creating one (`allow_create = false`), and
    /// checks only its length (the key itself is never printed):
    /// `cargo test reads_existing_device_key -- --ignored`
    #[test]
    #[ignore = "reads the real OS keychain; needs an existing device key"]
    fn reads_existing_device_key() {
        let key = super::resolve_device_key(false).expect("existing device key is found");
        assert_eq!(key.len(), 32);
    }
}
