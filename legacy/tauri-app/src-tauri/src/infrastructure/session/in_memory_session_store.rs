use std::collections::HashMap;
use std::sync::Mutex;

use sha2::{Digest, Sha256};

use crate::domain::entities::session::Session;
use crate::domain::services::session_store::SessionStore;

/// In-memory session store. Cleared when the process exits, which is exactly the
/// desired behaviour: a fresh app start has no sessions and forces re-login.
///
/// Sessions are keyed by the SHA-256 digest of the token and the stored copy has
/// its token redacted, so the raw token never sits in the map: a memory dump
/// reveals only digests, and lookups compare digests instead of the secret
/// itself (no timing signal on the token bytes).
#[derive(Default)]
pub struct InMemorySessionStore {
    sessions: Mutex<HashMap<[u8; 32], Session>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn key(token: &str) -> [u8; 32] {
        Sha256::digest(token.as_bytes()).into()
    }
}

impl SessionStore for InMemorySessionStore {
    fn insert(&self, session: Session) {
        let key = Self::key(&session.token);
        let redacted = Session {
            token: String::new(),
            ..session
        };
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(key, redacted);
    }

    fn get(&self, token: &str) -> Option<Session> {
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&Self::key(token))
            .cloned()
    }

    fn remove(&self, token: &str) {
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(&Self::key(token));
    }

    fn list(&self) -> Vec<Session> {
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .values()
            .cloned()
            .collect()
    }

    fn clear(&self) {
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
    }

    fn touch(&self, token: &str, now_ms: i64) {
        if let Some(s) = self
            .sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get_mut(&Self::key(token))
        {
            s.last_seen_ms = s.last_seen_ms.max(now_ms);
        }
    }

    fn touch_all(&self, now_ms: i64) {
        for s in self
            .sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .values_mut()
        {
            s.last_seen_ms = s.last_seen_ms.max(now_ms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(token: &str, expires_at: i64) -> Session {
        Session {
            token: token.to_string(),
            expires_at,
            last_seen_ms: 0,
            idle_timeout_ms: 1_000,
        }
    }

    #[test]
    fn round_trip_by_token_without_retaining_it() {
        let store = InMemorySessionStore::new();
        store.insert(session("tok-1", 42));

        let got = store.get("tok-1").expect("session found");
        assert_eq!(got.expires_at, 42);
        assert!(got.token.is_empty(), "raw token must not be retained");
        assert!(store.get("tok-2").is_none());

        store.remove("tok-1");
        assert!(store.get("tok-1").is_none());
    }

    #[test]
    fn list_and_clear() {
        let store = InMemorySessionStore::new();
        store.insert(session("a", 1));
        store.insert(session("b", 2));
        assert_eq!(store.list().len(), 2);

        store.clear();
        assert!(store.list().is_empty());
    }

    #[test]
    fn touch_records_activity() {
        let store = InMemorySessionStore::new();
        store.insert(session("a", 10_000));
        store.touch("a", 500);
        assert_eq!(store.get("a").unwrap().last_seen_ms, 500);
        store.touch_all(900);
        assert_eq!(store.get("a").unwrap().last_seen_ms, 900);
        // Never moves backwards.
        store.touch("a", 100);
        assert_eq!(store.get("a").unwrap().last_seen_ms, 900);
    }
}
