//! Session lifetime on the UI side (port of `use-session-timer.ts` and
//! `use-idle-timeout.ts`): the absolute expiry countdown, the inactivity
//! logout, and the activity pings that keep the backend's own idle timer in
//! step with a user who is active without triggering any backend call.
//!
//! Everything is driven by wall-clock epoch-ms (not a decrementing counter),
//! so sleep or suspend can never grant extra session time.

use std::sync::{Arc, Mutex};

use zeroize::Zeroizing;

/// Inactivity logout (the backend locks the vault after its own 6 min).
pub const IDLE_TIMEOUT_MS: i64 = 5 * 60_000;
/// The countdown turns red under this.
pub const WARNING_MS: i64 = 60_000;
/// At most one activity ping to the backend per this interval.
pub const ACTIVITY_PING_MS: i64 = 60_000;

/// Pure timing rules of one session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionClock {
    pub expires_at: i64,
    pub last_activity: i64,
    pub last_ping: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tick {
    Running {
        remaining_ms: i64,
    },
    /// Absolute lifetime reached.
    Expired,
    /// No user activity for [`IDLE_TIMEOUT_MS`].
    Idle,
}

impl SessionClock {
    pub fn new(expires_at: i64, now: i64) -> Self {
        Self {
            expires_at,
            last_activity: now,
            last_ping: now,
        }
    }

    pub fn tick(&self, now: i64) -> Tick {
        if now >= self.expires_at {
            Tick::Expired
        } else if now - self.last_activity >= IDLE_TIMEOUT_MS {
            Tick::Idle
        } else {
            Tick::Running {
                remaining_ms: self.expires_at - now,
            }
        }
    }

    /// Record user activity; `true` when the backend should be pinged.
    pub fn activity(&mut self, now: i64) -> bool {
        self.last_activity = now;
        if now - self.last_ping >= ACTIVITY_PING_MS {
            self.last_ping = now;
            true
        } else {
            false
        }
    }
}

struct Active {
    token: Zeroizing<String>,
    clock: SessionClock,
}

/// The signed-in session, if any. The token lives in Rust memory only (never
/// in the UI, never on disk) and is zeroized when the session ends.
#[derive(Clone, Default)]
pub struct Session(Arc<Mutex<Option<Active>>>);

impl Session {
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Active>> {
        self.0.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn start(&self, token: String, expires_at: i64, now: i64) {
        *self.lock() = Some(Active {
            token: Zeroizing::new(token),
            clock: SessionClock::new(expires_at, now),
        });
    }

    /// End the session locally, handing back its token (for the backend
    /// logout).
    pub fn clear(&self) -> Option<Zeroizing<String>> {
        self.lock().take().map(|a| a.token)
    }

    pub fn is_active(&self) -> bool {
        self.lock().is_some()
    }

    pub fn tick(&self, now: i64) -> Option<Tick> {
        self.lock().as_ref().map(|a| a.clock.tick(now))
    }

    /// Record activity; returns the token when a backend ping is due.
    pub fn activity(&self, now: i64) -> Option<Zeroizing<String>> {
        let mut guard = self.lock();
        let active = guard.as_mut()?;
        active.clock.activity(now).then(|| active.token.clone())
    }
}

/// Wall-clock epoch milliseconds.
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn countdown_then_absolute_expiry() {
        let c = SessionClock::new(900_000, 0);
        assert_eq!(
            c.tick(0),
            Tick::Running {
                remaining_ms: 900_000
            }
        );
        let mut c = c;
        // Stay active so the idle rule does not fire first.
        for t in (0..900_000).step_by(60_000) {
            c.activity(t);
        }
        assert_eq!(c.tick(899_999), Tick::Running { remaining_ms: 1 });
        assert_eq!(c.tick(900_000), Tick::Expired);
    }

    #[test]
    fn inactivity_ends_the_session() {
        let mut c = SessionClock::new(900_000, 0);
        assert_eq!(
            c.tick(IDLE_TIMEOUT_MS - 1),
            Tick::Running {
                remaining_ms: 900_000 - IDLE_TIMEOUT_MS + 1
            }
        );
        assert_eq!(c.tick(IDLE_TIMEOUT_MS), Tick::Idle);
        c.activity(IDLE_TIMEOUT_MS - 10);
        assert!(matches!(
            c.tick(IDLE_TIMEOUT_MS + 1000),
            Tick::Running { .. }
        ));
    }

    #[test]
    fn activity_pings_are_throttled() {
        let mut c = SessionClock::new(900_000, 0);
        assert!(!c.activity(10));
        assert!(!c.activity(59_999));
        assert!(c.activity(60_000));
        assert!(!c.activity(61_000));
        assert!(c.activity(120_000));
    }

    #[test]
    fn session_token_is_handed_back_once() {
        let s = Session::default();
        assert!(!s.is_active());
        assert_eq!(s.tick(0), None);
        s.start("tok".into(), 1000, 0);
        assert!(s.is_active());
        assert_eq!(
            s.activity(60_000).as_deref().map(String::as_str),
            Some("tok")
        );
        assert_eq!(s.clear().as_deref().map(String::as_str), Some("tok"));
        assert!(s.clear().is_none());
    }
}
