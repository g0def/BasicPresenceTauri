//! UI language. The strings are bundled at build time (see `build.rs`):
//! Slint's `@tr` for the static texts of the UI, and [`tr`] / [`tr_with`]
//! for the texts Rust computes. Both come from `i18n/{fr,en}.json`.

pub use basic_presence_core::application::view::format::Lang;

mod strings {
    include!(concat!(env!("OUT_DIR"), "/strings.rs"));
}

/// Translation of `key` (the key itself if unknown).
pub fn tr(lang: Lang, key: &str) -> String {
    lookup(lang, key).unwrap_or(key).to_string()
}

/// Translation of `key`, or `None` if the key does not exist.
pub fn lookup(lang: Lang, key: &str) -> Option<&'static str> {
    let table = match lang {
        Lang::Fr => strings::FR,
        Lang::En => strings::EN,
    };
    table
        .binary_search_by(|(k, _)| (*k).cmp(key))
        .ok()
        .map(|i| table[i].1)
}

/// Translation with `{{name}}` placeholders filled in.
pub fn tr_with(lang: Lang, key: &str, args: &[(&str, &str)]) -> String {
    let mut text = tr(lang, key);
    for (name, value) in args {
        text = text.replace(&format!("{{{{{name}}}}}"), value);
    }
    text
}

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
    use super::*;

    #[test]
    fn rust_side_translations() {
        assert_eq!(tr(Lang::Fr, "presence.types.office"), "Bureau");
        assert_eq!(tr(Lang::En, "presence.types.office"), "Office");
        assert_eq!(tr(Lang::Fr, "no.such.key"), "no.such.key");
        assert_eq!(lookup(Lang::En, "no.such.key"), None);
        assert_eq!(
            tr_with(Lang::En, "profile.deleteConfirmBody", &[("name", "Ada")]),
            "The profile \"Ada\" will be permanently deleted."
        );
    }

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
