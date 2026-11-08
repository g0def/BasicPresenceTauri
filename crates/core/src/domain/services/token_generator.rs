use crate::domain::error::DomainError;

/// Port for generating cryptographically-random session tokens.
pub trait TokenGenerator: Send + Sync {
    fn generate(&self) -> Result<String, DomainError>;
}
