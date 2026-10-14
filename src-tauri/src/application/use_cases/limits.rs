//! Size bounds for untrusted input.
//!
//! Everything reachable from the IPC boundary or from an imported bundle is
//! attacker-controlled in practice: a bundle is a plain `.json` a colleague
//! sends over, and the WebView is only as trustworthy as its dependencies.
//! Without bounds, a single file can carry a 200 MB note or a million day
//! entries — each of which is then held in memory, written to the vault, and
//! re-read on every load.
//!
//! These are deliberately generous: they are a backstop against absurd input,
//! not a product constraint. Anything a human would actually type fits.

use crate::domain::error::DomainError;

/// Short identity-like fields (names, enterprise, job title).
pub(crate) const MAX_FIELD_LEN: usize = 120;

/// Free-text one-liners (task titles, commute names).
pub(crate) const MAX_TITLE_LEN: usize = 200;

/// Free-text paragraphs (task descriptions).
pub(crate) const MAX_DESCRIPTION_LEN: usize = 2_000;

/// A day's Markdown note. Roughly 100 pages of prose.
pub(crate) const MAX_NOTE_LEN: usize = 200_000;

/// Bundle file size on disk, before parsing. A real profile with ten years of
/// daily entries and notes stays far below this.
pub(crate) const MAX_BUNDLE_BYTES: u64 = 32 * 1024 * 1024;

/// Per-category item counts inside a bundle. ~137 years of daily presence.
pub(crate) const MAX_BUNDLE_DAYS: usize = 50_000;
/// Presets, commutes, and per-day work entries / trips.
pub(crate) const MAX_BUNDLE_ITEMS: usize = 10_000;

/// Reject a string longer than `max` characters (counted as chars, not bytes,
/// so the limit does not depend on the alphabet used).
pub(crate) fn validate_len(field: &str, value: &str, max: usize) -> Result<(), DomainError> {
    if value.chars().count() > max {
        return Err(DomainError::Validation(format!(
            "{field} must be at most {max} characters"
        )));
    }
    Ok(())
}

/// Reject a collection larger than `max` items.
pub(crate) fn validate_count(field: &str, len: usize, max: usize) -> Result<(), DomainError> {
    if len > max {
        return Err(DomainError::Validation(format!(
            "{field} must contain at most {max} items (got {len})"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_characters_not_bytes() {
        // 4 chars, 8 bytes in UTF-8: a byte-based limit would reject this.
        assert!(validate_len("field", "éàçü", 4).is_ok());
        assert!(validate_len("field", "éàçü", 3).is_err());
    }

    #[test]
    fn bounds_collections() {
        assert!(validate_count("days", 10, 10).is_ok());
        assert!(validate_count("days", 11, 10).is_err());
    }
}
