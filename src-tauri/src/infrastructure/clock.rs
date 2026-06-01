use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::services::clock::Clock;

/// Wall-clock based on the system time, in epoch milliseconds.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
}
