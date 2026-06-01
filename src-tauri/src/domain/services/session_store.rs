use crate::domain::entities::session::Session;

/// In-memory store of active sessions. It is intentionally "dumb" storage —
/// expiry logic lives in the use cases (which own the `Clock`).
pub trait SessionStore: Send + Sync {
    fn insert(&self, session: Session);
    fn get(&self, token: &str) -> Option<Session>;
    fn remove(&self, token: &str);
}
