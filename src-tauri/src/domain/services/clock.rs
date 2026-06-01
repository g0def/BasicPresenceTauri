/// Port for reading the current time (epoch ms) — injectable for testing.
pub trait Clock: Send + Sync {
    fn now_ms(&self) -> i64;
}
