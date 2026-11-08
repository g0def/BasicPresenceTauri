/// In-memory session. Never persisted to disk, which forces re-authentication
/// at every app start.
#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    /// Absolute expiry, epoch milliseconds.
    pub expires_at: i64,
    /// Last backend activity (session-gated command or session check).
    pub last_seen_ms: i64,
    /// Inactivity after which the session is no longer valid.
    pub idle_timeout_ms: i64,
}

impl Session {
    /// Valid while neither the absolute lifetime nor the idle timeout elapsed.
    pub fn is_valid(&self, now_ms: i64) -> bool {
        self.expires_at > now_ms && now_ms - self.last_seen_ms < self.idle_timeout_ms
    }

    /// Time left before the absolute expiry (what the UI counts down).
    pub fn remaining_ms(&self, now_ms: i64) -> i64 {
        (self.expires_at - now_ms).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session {
        Session {
            token: String::new(),
            expires_at: 10_000,
            last_seen_ms: 0,
            idle_timeout_ms: 1_000,
        }
    }

    #[test]
    fn idle_timeout_ends_a_session_before_its_absolute_expiry() {
        let s = session();
        assert!(s.is_valid(999));
        assert!(!s.is_valid(1_000));
    }

    #[test]
    fn activity_does_not_extend_the_absolute_expiry() {
        let s = Session {
            last_seen_ms: 9_999,
            ..session()
        };
        assert!(s.is_valid(9_999));
        assert!(!s.is_valid(10_000));
    }
}
