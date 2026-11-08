//! Markdown rendered as a flat list of display blocks.
//!
//! The native UI has no HTML engine: instead of sanitized HTML, the backend
//! hands out a structured, inert representation (text runs + styles) that the
//! UI draws with its own widgets. Nothing in it is ever interpreted as markup,
//! so embedded HTML simply shows up as literal text; link targets are vetted
//! here (only `http`, `https` and `mailto` survive).

use serde::Serialize;

/// A run of inline text sharing one style.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdSpan {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub strike: bool,
    /// Inline `code`.
    pub code: bool,
    /// Vetted link target (`http`, `https` or `mailto` only).
    pub link: Option<String>,
}

/// One syntax-highlighted token of a code block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeToken {
    pub text: String,
    /// Foreground color, `0xRRGGBB`.
    pub rgb: u32,
}

/// Leading marker of a list item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum ListMarker {
    Bullet,
    /// Ordered item with its number.
    Ordered(u64),
    /// GFM task item, checked or not.
    Task(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum MdBlockKind {
    Heading {
        /// 1 to 6.
        level: u8,
        spans: Vec<MdSpan>,
    },
    Paragraph {
        spans: Vec<MdSpan>,
    },
    ListItem {
        marker: ListMarker,
        spans: Vec<MdSpan>,
    },
    Code {
        lang: Option<String>,
        /// Theme background, `0xRRGGBB` (a self-contained dark island that
        /// stays legible under both the light and dark UI).
        background: u32,
        lines: Vec<Vec<CodeToken>>,
    },
    Rule,
    Table {
        header: Vec<Vec<MdSpan>>,
        rows: Vec<Vec<Vec<MdSpan>>>,
    },
}

/// One display block. Nesting is flattened into depths so a UI can render the
/// whole document as a simple vertical list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdBlock {
    /// List nesting level (0 = top level).
    pub indent: u8,
    /// Block-quote nesting level (0 = not quoted).
    pub quote_depth: u8,
    pub kind: MdBlockKind,
}

/// Port: Markdown source → display blocks.
pub trait MarkdownRenderer: Send + Sync {
    fn render_blocks(&self, markdown: &str) -> Vec<MdBlock>;
}

/// Whether a link target may be exposed to the UI (opened in the browser).
pub fn is_safe_link(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    ["http://", "https://", "mailto:"]
        .iter()
        .any(|scheme| lower.starts_with(scheme))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_web_and_mail_links_are_safe() {
        assert!(is_safe_link("https://example.org"));
        assert!(is_safe_link("HTTP://example.org"));
        assert!(is_safe_link("mailto:a@b.c"));
        assert!(!is_safe_link("javascript:alert(1)"));
        assert!(!is_safe_link("file:///etc/passwd"));
        assert!(!is_safe_link("relative/path"));
        assert!(!is_safe_link(" data:text/html,x"));
    }
}
