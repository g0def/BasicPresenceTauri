use argon2::password_hash::{
    Error as PhError, PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString,
};
use argon2::{Algorithm, Argon2, Params, Version};

use crate::domain::error::DomainError;
use crate::domain::services::password_hasher::PasswordHasher;

/// Argon2id password hasher (auth). Produces/verifies PHC strings.
pub struct Argon2PasswordHasher {
    params: Params,
}

impl Argon2PasswordHasher {
    pub fn new(m_cost: u32, t_cost: u32, p_cost: u32) -> Result<Self, DomainError> {
        let params = Params::new(m_cost, t_cost, p_cost, None)
            .map_err(|e| DomainError::Hashing(e.to_string()))?;
        Ok(Self { params })
    }

    fn argon2(&self) -> Argon2<'_> {
        Argon2::new(Algorithm::Argon2id, Version::V0x13, self.params.clone())
    }
}

impl PasswordHasher for Argon2PasswordHasher {
    fn hash(&self, password: &[u8]) -> Result<String, DomainError> {
        let mut salt_bytes = [0u8; 16];
        getrandom::fill(&mut salt_bytes).map_err(|e| DomainError::Hashing(e.to_string()))?;
        let salt =
            SaltString::encode_b64(&salt_bytes).map_err(|e| DomainError::Hashing(e.to_string()))?;
        let hash = self
            .argon2()
            .hash_password(password, &salt)
            .map_err(|e| DomainError::Hashing(e.to_string()))?;
        Ok(hash.to_string())
    }

    fn verify(&self, password: &[u8], phc: &str) -> Result<bool, DomainError> {
        let parsed = PasswordHash::new(phc).map_err(|e| DomainError::Hashing(e.to_string()))?;
        match self.argon2().verify_password(password, &parsed) {
            Ok(()) => Ok(true),
            Err(PhError::Password) => Ok(false),
            Err(e) => Err(DomainError::Hashing(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tiny params keep the test fast; production params live in AppConfig.
    fn hasher() -> Argon2PasswordHasher {
        Argon2PasswordHasher::new(64, 1, 1).unwrap()
    }

    #[test]
    fn hash_and_verify_roundtrip() {
        let h = hasher();
        let phc = h.hash(b"correct horse battery staple").unwrap();
        assert!(h.verify(b"correct horse battery staple", &phc).unwrap());
        assert!(!h.verify(b"wrong password", &phc).unwrap());
    }

    #[test]
    fn distinct_salts_produce_distinct_hashes() {
        let h = hasher();
        assert_ne!(h.hash(b"same").unwrap(), h.hash(b"same").unwrap());
    }
}
