use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{Key, KeyInit, XChaCha20Poly1305, XNonce};
use zeroize::{Zeroize, Zeroizing};

use crate::domain::error::DomainError;
use crate::domain::services::key_service::{KeyService, WrappedKey};

/// Envelope-encryption key service:
/// - KEK derived from the password via Argon2id,
/// - DEK wrapped with the KEK via XChaCha20-Poly1305 (authenticated AEAD).
pub struct Argon2KeyService {
    params: Params,
}

impl Argon2KeyService {
    pub fn new(m_cost: u32, t_cost: u32, p_cost: u32) -> Result<Self, DomainError> {
        // 32-byte output = the KEK length.
        let params = Params::new(m_cost, t_cost, p_cost, Some(32))
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(Self { params })
    }
}

impl KeyService for Argon2KeyService {
    fn derive_kek(&self, password: &[u8], salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, DomainError> {
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, self.params.clone());
        let mut out = Zeroizing::new([0u8; 32]);
        argon
            .hash_password_into(password, salt, out.as_mut_slice())
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(out)
    }

    fn generate_dek(&self) -> Result<Zeroizing<[u8; 32]>, DomainError> {
        let mut dek = Zeroizing::new([0u8; 32]);
        getrandom::fill(dek.as_mut_slice()).map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(dek)
    }

    fn generate_salt(&self) -> Result<Vec<u8>, DomainError> {
        let mut salt = vec![0u8; 16];
        getrandom::fill(&mut salt).map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(salt)
    }

    fn wrap_dek(&self, dek: &[u8], kek: &[u8]) -> Result<WrappedKey, DomainError> {
        if kek.len() != 32 {
            return Err(DomainError::Crypto("invalid KEK length".into()));
        }
        let cipher = XChaCha20Poly1305::new(Key::from_slice(kek));
        let mut nonce_bytes = [0u8; 24];
        getrandom::fill(&mut nonce_bytes).map_err(|e| DomainError::Crypto(e.to_string()))?;
        let nonce = XNonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, dek)
            .map_err(|_| DomainError::Crypto("failed to wrap DEK".into()))?;
        Ok(WrappedKey {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    fn unwrap_dek(
        &self,
        wrapped: &WrappedKey,
        kek: &[u8],
    ) -> Result<Zeroizing<[u8; 32]>, DomainError> {
        if kek.len() != 32 {
            return Err(DomainError::Crypto("invalid KEK length".into()));
        }
        if wrapped.nonce.len() != 24 {
            return Err(DomainError::Crypto("invalid nonce length".into()));
        }
        let cipher = XChaCha20Poly1305::new(Key::from_slice(kek));
        let nonce = XNonce::from_slice(&wrapped.nonce);
        let mut plaintext = cipher
            .decrypt(nonce, wrapped.ciphertext.as_ref())
            .map_err(|_| DomainError::Crypto("failed to unwrap DEK".into()))?;
        if plaintext.len() != 32 {
            plaintext.zeroize();
            return Err(DomainError::Crypto("unexpected DEK length".into()));
        }
        let mut dek = Zeroizing::new([0u8; 32]);
        dek.as_mut_slice().copy_from_slice(&plaintext);
        plaintext.zeroize();
        Ok(dek)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> Argon2KeyService {
        // Tiny Argon2 params keep the test fast.
        Argon2KeyService::new(64, 1, 1).unwrap()
    }

    #[test]
    fn wrap_then_unwrap_recovers_dek() {
        let ks = service();
        let dek = ks.generate_dek().unwrap();
        let salt = ks.generate_salt().unwrap();
        let kek = ks.derive_kek(b"hunter2", &salt).unwrap();

        let wrapped = ks.wrap_dek(dek.as_slice(), kek.as_slice()).unwrap();
        let unwrapped = ks.unwrap_dek(&wrapped, kek.as_slice()).unwrap();

        assert_eq!(dek.as_slice(), unwrapped.as_slice());
    }

    #[test]
    fn wrong_password_fails_to_unwrap() {
        let ks = service();
        let dek = ks.generate_dek().unwrap();
        let salt = ks.generate_salt().unwrap();
        let kek = ks.derive_kek(b"hunter2", &salt).unwrap();
        let wrapped = ks.wrap_dek(dek.as_slice(), kek.as_slice()).unwrap();

        let bad_kek = ks.derive_kek(b"not-the-password", &salt).unwrap();
        assert!(ks.unwrap_dek(&wrapped, bad_kek.as_slice()).is_err());
    }

    #[test]
    fn same_password_different_salt_yields_different_kek() {
        let ks = service();
        let s1 = ks.generate_salt().unwrap();
        let s2 = ks.generate_salt().unwrap();
        let k1 = ks.derive_kek(b"pw", &s1).unwrap();
        let k2 = ks.derive_kek(b"pw", &s2).unwrap();
        assert_ne!(k1.as_slice(), k2.as_slice());
    }
}
