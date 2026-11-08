use async_trait::async_trait;

use crate::domain::entities::commute::Commute;
use crate::domain::error::DomainError;

/// Persistence contract for saved commute templates (in the vault). A commute
/// owns its ordered segments; create/update persist both atomically.
#[async_trait]
pub trait CommuteRepository: Send + Sync {
    async fn create(&self, commute: &Commute) -> Result<(), DomainError>;

    async fn list_by_profile(&self, profile_id: &str) -> Result<Vec<Commute>, DomainError>;

    async fn find_by_id(&self, id: &str) -> Result<Option<Commute>, DomainError>;

    /// Replace the commute's fields and its full set of segments.
    async fn update(&self, commute: &Commute) -> Result<(), DomainError>;

    async fn delete(&self, id: &str) -> Result<(), DomainError>;
}
