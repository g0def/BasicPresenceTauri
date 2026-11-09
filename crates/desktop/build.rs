//! Compiles the Slint UI and bundles its translations.
//!
//! Source of truth for the strings: `i18n/{fr,en}.json` (the i18next files of
//! the former React app, nested keys). They are turned into gettext catalogs
//! `$OUT_DIR/translations/<lang>/LC_MESSAGES/basic-presence.po` whose msgids
//! are the dotted keys, so the UI writes `@tr("auth.password")`:
//!
//! - `{{name}}` placeholders become positional `{0}`, `{1}`… numbered by first
//!   appearance in the French string; pass the arguments in that order:
//!   `@tr("profile.deleteConfirmBody", name)`.
//! - i18next plurals (`key_one` / `key_other`) become one plural entry, used as
//!   `@tr("key_one" | "key_other" % n)`; `{{count}}` becomes `{n}`.
//!
//! The build fails if fr and en do not have the same keys, or if a `.slint`
//! file uses a key that does not exist (Slint would silently show the key).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const LANGS: [&str; 2] = ["fr", "en"];
const DOMAIN: &str = "basic-presence";

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("translations");

    let catalogs: BTreeMap<&str, BTreeMap<String, String>> = LANGS
        .iter()
        .map(|lang| {
            let path = manifest.join("i18n").join(format!("{lang}.json"));
            println!("cargo:rerun-if-changed={}", path.display());
            let json: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(&path).expect("read translation file"),
            )
            .expect("valid translation JSON");
            let mut flat = BTreeMap::new();
            flatten(&json, String::new(), &mut flat);
            (*lang, flat)
        })
        .collect();

    // Parity: every language has exactly the same keys.
    let reference: BTreeSet<&String> = catalogs["fr"].keys().collect();
    for (lang, flat) in &catalogs {
        let keys: BTreeSet<&String> = flat.keys().collect();
        let missing: Vec<_> = reference.difference(&keys).collect();
        let extra: Vec<_> = keys.difference(&reference).collect();
        assert!(
            missing.is_empty() && extra.is_empty(),
            "i18n/{lang}.json out of sync with fr: missing {missing:?}, extra {extra:?}"
        );
    }

    // Placeholder numbering, decided once from French for both languages.
    let placeholder_order: BTreeMap<&String, Vec<String>> = catalogs["fr"]
        .iter()
        .map(|(k, v)| (k, placeholders(v)))
        .collect();

    for (lang, flat) in &catalogs {
        let po = to_po(lang, flat, &placeholder_order);
        let dir = out.join(lang).join("LC_MESSAGES");
        std::fs::create_dir_all(&dir).expect("create translations dir");
        std::fs::write(dir.join(format!("{DOMAIN}.po")), po).expect("write .po");
    }

    check_slint_keys(&manifest.join("ui"), &catalogs["fr"]);

    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent".into())
        .with_bundled_translations(&out)
        .with_default_translation_context(slint_build::DefaultTranslationContext::None);
    slint_build::compile_with_config("ui/app.slint", config).expect("Slint build failed");
}

/// Nested JSON objects → dotted keys.
fn flatten(value: &serde_json::Value, prefix: String, out: &mut BTreeMap<String, String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten(v, key, out);
            }
        }
        serde_json::Value::String(s) => {
            out.insert(prefix, s.clone());
        }
        other => panic!("translation {prefix}: unsupported value {other}"),
    }
}

/// `{{name}}` placeholder names, in order of first appearance.
fn placeholders(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let end = after.find("}}").expect("unclosed {{ placeholder");
        let name = after[..end].trim().to_string();
        if !names.contains(&name) {
            names.push(name);
        }
        rest = &after[end + 2..];
    }
    names
}

/// Rewrite i18next placeholders into Slint's format syntax.
fn convert(text: &str, order: &[String], plural: bool) -> String {
    let mut s = text.to_string();
    for (i, name) in order.iter().enumerate() {
        let target = if plural && name == "count" {
            "{n}".to_string()
        } else {
            format!("{{{i}}}")
        };
        s = s.replace(&format!("{{{{{name}}}}}"), &target);
    }
    assert!(
        !s.contains("{{"),
        "placeholder of {text:?} missing from the French string"
    );
    s
}

fn po_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

fn to_po(
    lang: &str,
    flat: &BTreeMap<String, String>,
    order: &BTreeMap<&String, Vec<String>>,
) -> String {
    // French: 0 and 1 are singular; English: only 1.
    let plural_forms = match lang {
        "fr" => "nplurals=2; plural=(n > 1);",
        _ => "nplurals=2; plural=(n != 1);",
    };
    let mut po = String::new();
    writeln!(
        po,
        "msgid \"\"\nmsgstr \"\"\n\"Content-Type: text/plain; charset=UTF-8\\n\"\n\"Language: {lang}\\n\"\n\"Plural-Forms: {plural_forms}\\n\"\n"
    )
    .unwrap();
    for (key, value) in flat {
        if let Some(base) = key.strip_suffix("_one") {
            let other_key = format!("{base}_other");
            let other = flat
                .get(&other_key)
                .unwrap_or_else(|| panic!("{key} has no {other_key}"));
            writeln!(
                po,
                "msgid \"{}\"\nmsgid_plural \"{}\"\nmsgstr[0] \"{}\"\nmsgstr[1] \"{}\"\n",
                po_escape(key),
                po_escape(&other_key),
                po_escape(&convert(value, &order[key], true)),
                po_escape(&convert(other, &order[&other_key], true)),
            )
            .unwrap();
        } else if key.ends_with("_other") {
            // Emitted with its `_one` sibling.
        } else {
            writeln!(
                po,
                "msgid \"{}\"\nmsgstr \"{}\"\n",
                po_escape(key),
                po_escape(&convert(value, &order[key], false))
            )
            .unwrap();
        }
    }
    po
}

/// Fail the build when a `.slint` file uses an unknown translation key.
fn check_slint_keys(dir: &Path, keys: &BTreeMap<String, String>) {
    for entry in std::fs::read_dir(dir).expect("read ui dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            check_slint_keys(&path, keys);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("slint") {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let source = std::fs::read_to_string(&path).expect("read .slint");
        let mut rest = source.as_str();
        while let Some(i) = rest.find("@tr(") {
            rest = &rest[i + 4..];
            // Every string literal up to the matching `)` or `,` / `%`: the
            // msgid and, for plurals, the msgid_plural.
            let call_end = rest.find([')', ',', '%']).unwrap_or(rest.len());
            let call = &rest[..call_end];
            for literal in call.split('"').skip(1).step_by(2) {
                assert!(
                    keys.contains_key(literal),
                    "{}: unknown translation key {literal:?}",
                    path.display()
                );
            }
        }
    }
}
