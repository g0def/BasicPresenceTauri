use crate::domain::entities::session::Session;

/// In-memory store of active sessions. It is intentionally "dumb" storage —
/// expiry logic lives in the use cases (which own the `Clock`).
///
/// Implementations must never retain the raw token at rest: sessions returned
/// by `get`/`list` carry a redacted (empty) token.
pub trait SessionStore: Send + Sync {
    fn insert(&self, session: Session);
    fn get(&self, token: &str) -> Option<Session>;
    fn remove(&self, token: &str);
    /// Snapshot of every stored session (tokens redacted).
    fn list(&self) -> Vec<Session>;
    /// Drop every stored session.
    fn clear(&self);
}
