use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;

use crate::domain::error::DomainError;

/// A unit of work: the repository calls it awaits.
pub type Work<'a> = Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + 'a>>;

/// Port running several repository operations as one atomic transaction: every
/// write made while `work` runs commits together, or none does.
///
/// Use it whenever a use case writes through more than one repository call
/// (e.g. a presence and its schedule, a profile and the active-profile marker,
/// a whole bundle import), so a failure halfway cannot leave a state the rest
/// of the app deems impossible.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn run<'a>(&self, work: Work<'a>) -> Result<(), DomainError>;
}
