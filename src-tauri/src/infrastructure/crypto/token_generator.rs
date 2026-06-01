use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

use crate::domain::error::DomainError;
use crate::domain::services::token_generator::TokenGenerator;

/// Generates 256-bit cryptographically-random, URL-safe session tokens.
pub struct RandomTokenGenerator;

impl TokenGenerator for RandomTokenGenerator {
    fn generate(&self) -> Result<String, DomainError> {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|e| DomainError::Token(e.to_string()))?;
        Ok(URL_SAFE_NO_PAD.encode(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_unique_and_nonempty() {
        let g = RandomTokenGenerator;
        let a = g.generate().unwrap();
        let b = g.generate().unwrap();
        assert!(!a.is_empty());
        assert_ne!(a, b);
    }
}
