use async_trait::async_trait;

use crate::domain::entities::emission_factor::{EmissionFactor, GridVariant};
use crate::domain::error::DomainError;

/// Read-only access to the versioned emission-factor referential (in the vault).
#[async_trait]
pub trait EmissionFactorRepository: Send + Sync {
    /// All factors for a reference year.
    async fn list(&self, year: i32) -> Result<Vec<EmissionFactor>, DomainError>;

    /// All per-country electric grid overrides for a reference year.
    async fn list_grid_variants(&self, year: i32) -> Result<Vec<GridVariant>, DomainError>;
}
