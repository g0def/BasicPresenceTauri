use std::collections::HashMap;
use std::sync::Mutex;

use crate::domain::entities::session::Session;
use crate::domain::services::session_store::SessionStore;

/// In-memory session store. Cleared when the process exits, which is exactly the
/// desired behaviour: a fresh app start has no sessions and forces re-login.
#[derive(Default)]
pub struct InMemorySessionStore {
    sessions: Mutex<HashMap<String, Session>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SessionStore for InMemorySessionStore {
    fn insert(&self, session: Session) {
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(session.token.clone(), session);
    }

    fn get(&self, token: &str) -> Option<Session> {
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(token)
            .cloned()
    }

    fn remove(&self, token: &str) {
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(token);
    }
}
