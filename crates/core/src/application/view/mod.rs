//! View-model helpers shared by any UI: pure functions that shape backend data
//! for display (formatting, chart buckets, geometry, wizard selections).
//!
//! Ported from the former React frontend (`legacy/tauri-app/src`) so the
//! native UI only binds data, and the logic stays unit-tested in Rust.

pub mod bundle_selection;
pub mod donut;
pub mod format;
pub mod methodology;
pub mod period_stats;
