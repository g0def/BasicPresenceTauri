use argon2::password_hash::{
    Error as PhError, PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString,
};
use argon2::{Algorithm, Argon2, Params, Version};
use async_trait::async_trait;
use zeroize::Zeroizing;

use crate::domain::error::DomainError;
use crate::domain::policy::KdfParams;
use crate::domain::services::password_hasher::PasswordHasher;
use crate::infrastructure::crypto::run_blocking;

/// Argon2id password hasher (auth). Produces/verifies PHC strings.
pub struct Argon2PasswordHasher {
    params: Params,
}

impl Argon2PasswordHasher {
    pub fn new(kdf: KdfParams) -> Result<Self, DomainError> {
        let params = Params::new(kdf.m_cost, kdf.t_cost, kdf.p_cost, None)
            .map_err(|e| DomainError::Hashing(e.to_string()))?;
        Ok(Self { params })
    }
}

fn argon2(params: Params) -> Argon2<'static> {
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Parameters embedded in a PHC string, if it is a well-formed Argon2 one.
fn phc_params(parsed: &PasswordHash<'_>) -> Option<KdfParams> {
    let params = Params::try_from(parsed).ok()?;
    Some(KdfParams {
        m_cost: params.m_cost(),
        t_cost: params.t_cost(),
        p_cost: params.p_cost(),
    })
}

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, password: &[u8]) -> Result<String, DomainError> {
        let mut salt_bytes = [0u8; 16];
        getrandom::fill(&mut salt_bytes).map_err(|e| DomainError::Hashing(e.to_string()))?;
        let salt =
            SaltString::encode_b64(&salt_bytes).map_err(|e| DomainError::Hashing(e.to_string()))?;
        let password = Zeroizing::new(password.to_vec());
        let params = self.params.clone();
        run_blocking(move || {
            argon2(params)
                .hash_password(&password, &salt)
                .map(|hash| hash.to_string())
                .map_err(|e| DomainError::Hashing(e.to_string()))
        })
        .await
    }

    async fn verify(&self, password: &[u8], phc: &str) -> Result<bool, DomainError> {
        let parsed = PasswordHash::new(phc).map_err(|e| DomainError::Hashing(e.to_string()))?;
        // The PHC string carries its own cost parameters: refuse absurd ones
        // before spending any memory/CPU on them.
        match phc_params(&parsed) {
            Some(p) if p.within_bounds() => {}
            _ => {
                return Err(DomainError::Hashing(
                    "stored hash parameters out of bounds".into(),
                ));
            }
        }
        let password = Zeroizing::new(password.to_vec());
        let phc = phc.to_string();
        let params = self.params.clone();
        run_blocking(move || {
            let parsed =
                PasswordHash::new(&phc).map_err(|e| DomainError::Hashing(e.to_string()))?;
            match argon2(params).verify_password(&password, &parsed) {
                Ok(()) => Ok(true),
                Err(PhError::Password) => Ok(false),
                Err(e) => Err(DomainError::Hashing(e.to_string())),
            }
        })
        .await
    }

    fn needs_rehash(&self, phc: &str) -> bool {
        let Ok(parsed) = PasswordHash::new(phc) else {
            return true;
        };
        phc_params(&parsed).is_none_or(|p| {
            p.m_cost != self.params.m_cost()
                || p.t_cost != self.params.t_cost()
                || p.p_cost != self.params.p_cost()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tiny params keep the test fast; production params live in AppConfig.
    fn hasher(m_cost: u32) -> Argon2PasswordHasher {
        Argon2PasswordHasher::new(KdfParams {
            m_cost,
            t_cost: 1,
            p_cost: 1,
        })
        .unwrap()
    }

    #[tokio::test]
    async fn hash_and_verify_roundtrip() {
        let h = hasher(64);
        let phc = h.hash(b"correct horse battery staple").await.unwrap();
        assert!(
            h.verify(b"correct horse battery staple", &phc)
                .await
                .unwrap()
        );
        assert!(!h.verify(b"wrong password", &phc).await.unwrap());
    }

    #[tokio::test]
    async fn distinct_salts_produce_distinct_hashes() {
        let h = hasher(64);
        assert_ne!(
            h.hash(b"same").await.unwrap(),
            h.hash(b"same").await.unwrap()
        );
    }

    #[tokio::test]
    async fn out_of_bounds_parameters_are_refused_without_hashing() {
        let h = hasher(64);
        let phc = h.hash(b"pw").await.unwrap();
        let forged = phc.replace("m=64", "m=4194304");
        assert!(h.verify(b"pw", &forged).await.is_err());
    }

    #[tokio::test]
    async fn rehash_is_needed_when_parameters_change() {
        let phc = hasher(64).hash(b"pw").await.unwrap();
        assert!(!hasher(64).needs_rehash(&phc));
        assert!(hasher(128).needs_rehash(&phc));
    }
}
