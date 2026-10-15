use argon2::{Algorithm, Argon2, Params, Version};
use async_trait::async_trait;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce};
use zeroize::{Zeroize, Zeroizing};

use crate::domain::error::DomainError;
use crate::domain::policy::KdfParams;
use crate::domain::services::key_service::{
    CURRENT_WRAP_VERSION, KeyBinding, KeyRole, KeyService, WrappedKey,
};
use crate::infrastructure::crypto::run_blocking;

/// Envelope-encryption key service:
/// - KEK derived from the password via Argon2id,
/// - keys wrapped with the KEK via XChaCha20-Poly1305 (authenticated AEAD),
///   with associated data binding each wrapped key to its role and account.
pub struct Argon2KeyService;

/// Associated data of a v2 wrap: `"bp-wrap-v2" ‖ 0x00 ‖ role ‖ 0x00 ‖ account_id`.
/// Swapping the wrapped DEK and MAC key, or transplanting key material from
/// another account, makes the AEAD tag check fail.
fn associated_data(binding: KeyBinding<'_>) -> Vec<u8> {
    let role: &[u8] = match binding.role {
        KeyRole::Dek => b"dek",
        KeyRole::VaultMac => b"vault-mac",
    };
    let mut aad = Vec::with_capacity(16 + role.len() + binding.account_id.len());
    aad.extend_from_slice(b"bp-wrap-v2");
    aad.push(0);
    aad.extend_from_slice(role);
    aad.push(0);
    aad.extend_from_slice(binding.account_id.as_bytes());
    aad
}

fn check_kek(kek: &[u8]) -> Result<XChaCha20Poly1305, DomainError> {
    if kek.len() != 32 {
        return Err(DomainError::Crypto("invalid KEK length".into()));
    }
    Ok(XChaCha20Poly1305::new(Key::from_slice(kek)))
}

#[async_trait]
impl KeyService for Argon2KeyService {
    async fn derive_kek(
        &self,
        password: &[u8],
        salt: &[u8],
        params: KdfParams,
    ) -> Result<Zeroizing<[u8; 32]>, DomainError> {
        if !params.within_bounds() {
            return Err(DomainError::Crypto("KDF parameters out of bounds".into()));
        }
        // 32-byte output = the KEK length.
        let params = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(32))
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        let password = Zeroizing::new(password.to_vec());
        let salt = salt.to_vec();
        run_blocking(move || {
            let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let mut out = Zeroizing::new([0u8; 32]);
            argon
                .hash_password_into(&password, &salt, out.as_mut_slice())
                .map_err(|e| DomainError::Crypto(e.to_string()))?;
            Ok(out)
        })
        .await
    }

    fn generate_key(&self) -> Result<Zeroizing<[u8; 32]>, DomainError> {
        let mut key = Zeroizing::new([0u8; 32]);
        getrandom::fill(key.as_mut_slice()).map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(key)
    }

    fn generate_salt(&self) -> Result<Vec<u8>, DomainError> {
        let mut salt = vec![0u8; 16];
        getrandom::fill(&mut salt).map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(salt)
    }

    fn wrap_key(
        &self,
        key: &[u8],
        kek: &[u8],
        role: KeyRole,
        account_id: &str,
    ) -> Result<WrappedKey, DomainError> {
        let cipher = check_kek(kek)?;
        let mut nonce_bytes = [0u8; 24];
        getrandom::fill(&mut nonce_bytes).map_err(|e| DomainError::Crypto(e.to_string()))?;
        let aad = associated_data(KeyBinding {
            version: CURRENT_WRAP_VERSION,
            role,
            account_id,
        });
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce_bytes),
                Payload {
                    msg: key,
                    aad: &aad,
                },
            )
            .map_err(|_| DomainError::Crypto("failed to wrap key".into()))?;
        Ok(WrappedKey {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    fn unwrap_key(
        &self,
        wrapped: &WrappedKey,
        kek: &[u8],
        binding: KeyBinding<'_>,
    ) -> Result<Zeroizing<[u8; 32]>, DomainError> {
        let cipher = check_kek(kek)?;
        if wrapped.nonce.len() != 24 {
            return Err(DomainError::Crypto("invalid nonce length".into()));
        }
        let aad = match binding.version {
            1 => Vec::new(),
            2 => associated_data(binding),
            _ => return Err(DomainError::Crypto("unknown key wrap version".into())),
        };
        let mut plaintext = cipher
            .decrypt(
                XNonce::from_slice(&wrapped.nonce),
                Payload {
                    msg: wrapped.ciphertext.as_ref(),
                    aad: &aad,
                },
            )
            .map_err(|_| DomainError::Crypto("failed to unwrap key".into()))?;
        if plaintext.len() != 32 {
            plaintext.zeroize();
            return Err(DomainError::Crypto("unexpected key length".into()));
        }
        let mut key = Zeroizing::new([0u8; 32]);
        key.as_mut_slice().copy_from_slice(&plaintext);
        plaintext.zeroize();
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tiny Argon2 params keep the test fast.
    const PARAMS: KdfParams = KdfParams {
        m_cost: 64,
        t_cost: 1,
        p_cost: 1,
    };

    fn binding(role: KeyRole, account_id: &str) -> KeyBinding<'_> {
        KeyBinding {
            version: CURRENT_WRAP_VERSION,
            role,
            account_id,
        }
    }

    async fn kek(password: &[u8], salt: &[u8]) -> Zeroizing<[u8; 32]> {
        Argon2KeyService
            .derive_kek(password, salt, PARAMS)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn wrap_then_unwrap_recovers_the_key() {
        let ks = Argon2KeyService;
        let dek = ks.generate_key().unwrap();
        let kek = kek(b"hunter2", &ks.generate_salt().unwrap()).await;
        let wrapped = ks
            .wrap_key(dek.as_slice(), kek.as_slice(), KeyRole::Dek, "acc")
            .unwrap();
        let unwrapped = ks
            .unwrap_key(&wrapped, kek.as_slice(), binding(KeyRole::Dek, "acc"))
            .unwrap();
        assert_eq!(dek.as_slice(), unwrapped.as_slice());
    }

    #[tokio::test]
    async fn wrong_password_fails_to_unwrap() {
        let ks = Argon2KeyService;
        let dek = ks.generate_key().unwrap();
        let salt = ks.generate_salt().unwrap();
        let good = kek(b"hunter2", &salt).await;
        let wrapped = ks
            .wrap_key(dek.as_slice(), good.as_slice(), KeyRole::Dek, "acc")
            .unwrap();
        let bad = kek(b"not-the-password", &salt).await;
        assert!(
            ks.unwrap_key(&wrapped, bad.as_slice(), binding(KeyRole::Dek, "acc"))
                .is_err()
        );
    }

    #[tokio::test]
    async fn wrapped_keys_are_bound_to_their_role_and_account() {
        let ks = Argon2KeyService;
        let key = ks.generate_key().unwrap();
        let kek = kek(b"pw", &ks.generate_salt().unwrap()).await;
        let wrapped = ks
            .wrap_key(key.as_slice(), kek.as_slice(), KeyRole::Dek, "acc")
            .unwrap();
        // Same KEK, but presented as the MAC key, or for another account.
        assert!(
            ks.unwrap_key(&wrapped, kek.as_slice(), binding(KeyRole::VaultMac, "acc"))
                .is_err()
        );
        assert!(
            ks.unwrap_key(&wrapped, kek.as_slice(), binding(KeyRole::Dek, "other"))
                .is_err()
        );
    }

    #[tokio::test]
    async fn legacy_v1_wraps_still_unwrap() {
        let ks = Argon2KeyService;
        let key = ks.generate_key().unwrap();
        let kek = kek(b"pw", &ks.generate_salt().unwrap()).await;
        // v1 = no associated data.
        let nonce = [3u8; 24];
        let ciphertext = check_kek(kek.as_slice())
            .unwrap()
            .encrypt(XNonce::from_slice(&nonce), key.as_slice())
            .unwrap();
        let wrapped = WrappedKey {
            ciphertext,
            nonce: nonce.to_vec(),
        };
        let legacy = KeyBinding {
            version: 1,
            role: KeyRole::Dek,
            account_id: "acc",
        };
        let unwrapped = ks.unwrap_key(&wrapped, kek.as_slice(), legacy).unwrap();
        assert_eq!(key.as_slice(), unwrapped.as_slice());
    }

    #[tokio::test]
    async fn same_password_different_salt_yields_different_kek() {
        let ks = Argon2KeyService;
        let k1 = kek(b"pw", &ks.generate_salt().unwrap()).await;
        let k2 = kek(b"pw", &ks.generate_salt().unwrap()).await;
        assert_ne!(k1.as_slice(), k2.as_slice());
    }
}
