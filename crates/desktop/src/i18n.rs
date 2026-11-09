//! UI language: the strings are bundled at build time (see `build.rs`); this
//! picks the active catalog.

/// Language of the OS session (`LC_ALL`, `LC_MESSAGES`, `LANG`), French by
/// default. Persisting the user's choice comes with the preferences (S6).
pub fn system_language() -> &'static str {
    let lang = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .find(|v| !v.is_empty())
        .unwrap_or_default();
    normalize(&lang)
}

/// `"en_US.UTF-8"` → `"en"`; anything that is not English → `"fr"`.
pub fn normalize(code: &str) -> &'static str {
    let code = code.trim().to_ascii_lowercase();
    if code == "en" || code.starts_with("en_") || code.starts_with("en-") || code.starts_with("en.")
    {
        "en"
    } else {
        "fr"
    }
}

/// Switch the bundled catalog; every `@tr` of the UI updates live.
pub fn select(lang: &str) {
    if let Err(e) = slint::select_bundled_translation(lang) {
        eprintln!("BasicPresence: cannot select language {lang}: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn normalizes_locales() {
        assert_eq!(normalize("en_US.UTF-8"), "en");
        assert_eq!(normalize("en"), "en");
        assert_eq!(normalize("EN-gb"), "en");
        assert_eq!(normalize("fr_BE.UTF-8"), "fr");
        assert_eq!(normalize("C"), "fr");
        assert_eq!(normalize(""), "fr");
        assert_eq!(normalize("eno"), "fr");
    }
}
