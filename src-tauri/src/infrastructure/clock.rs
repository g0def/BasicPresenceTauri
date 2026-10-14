use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::domain::services::clock::Clock;

/// Epoch-millisecond clock that never goes backwards.
///
/// Session expiry and the brute-force lockout are both deadline comparisons
/// against this clock, so a user who can set the system time could otherwise
/// extend a session or clear a lockout by moving it back.
///
/// Neither available time source is sufficient alone:
///
/// - the **wall clock** can jump in either direction (NTP correction, manual
///   change), and moving it back is exactly the abuse to stop;
/// - a **monotonic** reading (`Instant`) cannot be moved, but on Linux it does
///   not advance while the machine is suspended — a laptop closed for three
///   hours would come back with its session still alive.
///
/// Taking the maximum of the two covers both: a backwards wall-clock jump is
/// absorbed by the monotonic floor, and time spent suspended is picked up by the
/// wall clock. The result can therefore only move forward.
pub struct SystemClock {
    /// Wall-clock reading and monotonic instant captured at the same moment,
    /// plus the highest value handed out so far.
    anchor: Mutex<Anchor>,
}

struct Anchor {
    wall_ms_at_start: i64,
    started: Instant,
    last: i64,
}

fn wall_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl SystemClock {
    pub fn new() -> Self {
        let wall = wall_now_ms();
        Self {
            anchor: Mutex::new(Anchor {
                wall_ms_at_start: wall,
                started: Instant::now(),
                last: wall,
            }),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        let mut anchor = self.anchor.lock().unwrap_or_else(|p| p.into_inner());
        let monotonic = anchor
            .wall_ms_at_start
            .saturating_add(anchor.started.elapsed().as_millis() as i64);
        let candidate = wall_now_ms().max(monotonic);
        // Ratchet: never hand out a value below one already observed.
        anchor.last = anchor.last.max(candidate);
        anchor.last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_close_to_the_wall_clock() {
        let clock = SystemClock::new();
        let delta = (clock.now_ms() - wall_now_ms()).abs();
        assert!(
            delta < 1_000,
            "expected wall-clock agreement, off by {delta}"
        );
    }

    #[test]
    fn never_moves_backwards() {
        let clock = SystemClock::new();
        let mut previous = clock.now_ms();
        for _ in 0..1_000 {
            let now = clock.now_ms();
            assert!(now >= previous, "clock went backwards: {now} < {previous}");
            previous = now;
        }
    }

    /// The ratchet is what a backwards system-time change runs into: once a
    /// value has been observed, nothing below it is ever returned again.
    #[test]
    fn a_lower_reading_cannot_lower_the_result() {
        let clock = SystemClock::new();
        let observed = clock.now_ms();
        {
            let mut anchor = clock.anchor.lock().unwrap();
            // Simulate the wall clock being wound back an hour.
            anchor.wall_ms_at_start -= 3_600_000;
        }
        assert!(clock.now_ms() >= observed);
    }
}
