use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::options::Plugins;
use comrak::{markdown_to_html_with_plugins, Options};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Theme, ThemeSet};
use syntect::html::{styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::domain::services::markdown::MarkdownRenderer;

/// syntect theme used to colorize code blocks in the read-only view. A
/// self-contained dark "island" (its own background travels with it) so the
/// fixed token colors stay readable under both the light and dark app UIs.
const CODE_THEME: &str = "base16-ocean.dark";

/// Markdown → sanitized HTML renderer.
///
/// Renders CommonMark + common GFM extensions with raw inline HTML disabled
/// (`render.unsafe_` stays `false`, so any embedded HTML is escaped rather than
/// emitted), colorizes fenced code blocks server-side via syntect, then runs the
/// output through Ammonia's whitelist sanitizer (strips `<script>`/`<style>`/`on*`
/// handlers, vets URL schemes, adds `rel="noopener"`).
pub struct ComrakMarkdownRenderer {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl Default for ComrakMarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl ComrakMarkdownRenderer {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        // `get(..).unwrap_or_default()` rather than indexing: a missing theme key
        // must never panic the composition root (falls back to an uncolored theme).
        let theme = ThemeSet::load_defaults()
            .themes
            .get(CODE_THEME)
            .cloned()
            .unwrap_or_default();
        Self { syntax_set, theme }
    }
}

impl MarkdownRenderer for ComrakMarkdownRenderer {
    fn render_to_safe_html(&self, markdown: &str) -> String {
        let mut options = Options::default();
        options.extension.table = true;
        options.extension.strikethrough = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;

        let adapter = SyntectAdapter {
            syntax_set: &self.syntax_set,
            theme: &self.theme,
        };
        let mut plugins = Plugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&adapter);

        let rendered = markdown_to_html_with_plugins(markdown, &options, &plugins);

        // Ammonia's default whitelist is safe but drops (a) comrak's GFM task-list
        // checkbox `<input>` and (b) the inline `style` our highlighter emits.
        // Re-allow exactly those: comrak runs with `render.unsafe_=false`, so any
        // *authored* HTML/attribute was escaped and can't reach the sanitizer —
        // only our own trusted markup does.
        ammonia::Builder::default()
            .add_tags(["input"])
            .add_tag_attributes("input", ["type", "checked", "disabled"])
            .add_tag_attributes("pre", ["style"])
            .add_tag_attributes("span", ["style"])
            .clean(&rendered)
            .to_string()
    }
}

/// comrak code-fence highlighter backed by syntect (pure-Rust `fancy-regex`).
/// Emits inline-styled `<span>`s for the tokens and a `<pre>` carrying the
/// theme's background so the block is self-contained.
struct SyntectAdapter<'a> {
    syntax_set: &'a SyntaxSet,
    theme: &'a Theme,
}

impl SyntaxHighlighterAdapter for SyntectAdapter<'_> {
    fn write_highlighted(
        &self,
        output: &mut dyn fmt::Write,
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        let syntax = lang
            .filter(|l| !l.is_empty())
            .and_then(|l| self.syntax_set.find_syntax_by_token(l))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, self.theme);
        for line in LinesWithEndings::from(code) {
            let regions = highlighter
                .highlight_line(line, self.syntax_set)
                .map_err(|_| fmt::Error)?;
            let html = styled_line_to_highlighted_html(&regions, IncludeBackground::No)
                .map_err(|_| fmt::Error)?;
            output.write_str(&html)?;
        }
        Ok(())
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn fmt::Write,
        _attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        // Carry the theme's own background so the fixed token colors stay legible
        // regardless of the app's light/dark UI.
        let bg = self.theme.settings.background.unwrap_or(Color {
            r: 0x2b,
            g: 0x30,
            b: 0x3b,
            a: 0xff,
        });
        write!(
            output,
            "<pre style=\"background-color:#{:02x}{:02x}{:02x};padding:0.75rem;border-radius:0.5rem;overflow:auto\">",
            bg.r, bg.g, bg.b
        )
    }

    fn write_code_tag(
        &self,
        output: &mut dyn fmt::Write,
        _attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        output.write_str("<code>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(md: &str) -> String {
        ComrakMarkdownRenderer::new().render_to_safe_html(md)
    }

    #[test]
    fn renders_headings_and_emphasis() {
        let html = render("# Titre\n\nsome **bold** text");
        assert!(html.contains("<h1>Titre</h1>"), "got: {html}");
        assert!(html.contains("<strong>bold</strong>"), "got: {html}");
    }

    #[test]
    fn strips_script_tags() {
        let html = render("hello <script>alert(1)</script> world");
        assert!(!html.contains("<script"), "script survived: {html}");
        assert!(!html.contains("alert(1)") || !html.contains("<script"));
    }

    #[test]
    fn neutralizes_javascript_links() {
        let html = render("[click](javascript:alert(1))");
        assert!(!html.contains("javascript:"), "js scheme survived: {html}");
    }

    #[test]
    fn renders_gfm_extensions() {
        let html = render("- [x] done\n- [ ] todo");
        assert!(
            html.contains("type=\"checkbox\""),
            "tasklist missing: {html}"
        );
    }

    #[test]
    fn highlights_code_blocks() {
        let html = render("```rust\nfn main() {}\n```");
        assert!(html.contains("<pre style="), "themed <pre> missing: {html}");
        assert!(
            html.contains("<span style=\"color:"),
            "highlight spans missing: {html}"
        );
        // The sanitizer kept our inline styles but no stray script/handlers.
        assert!(!html.contains("<script"));
    }
}
