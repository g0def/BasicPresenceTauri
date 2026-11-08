pub mod commute_builder;
pub mod credentials;
pub mod factor_maps;

/// Saturating `usize → u32` for the counters reported to the UI.
pub fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}
