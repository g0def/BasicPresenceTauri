/// In-memory session. Never persisted to disk, which forces re-authentication
/// at every app start.
#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    /// Absolute expiry, epoch milliseconds.
    pub expires_at: i64,
}

impl Session {
    pub fn is_valid(&self, now_ms: i64) -> bool {
        self.expires_at > now_ms
    }

    pub fn remaining_ms(&self, now_ms: i64) -> i64 {
        (self.expires_at - now_ms).max(0)
    }
}
