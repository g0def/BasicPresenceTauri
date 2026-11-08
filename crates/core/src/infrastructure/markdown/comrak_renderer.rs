use comrak::nodes::{AstNode, ListType, NodeValue};
use comrak::{Arena, Options, parse_document};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::domain::services::markdown::{
    CodeToken, ListMarker, MarkdownRenderer, MdBlock, MdBlockKind, MdSpan, is_safe_link,
};

/// syntect theme used to colorize code blocks. A self-contained dark "island"
/// (its own background travels with it) so the fixed token colors stay
/// readable under both the light and dark app UIs.
const CODE_THEME: &str = "base16-ocean.dark";
/// Fallback code background (`base16-ocean.dark`).
const CODE_BG_FALLBACK: u32 = 0x2b303b;
/// Fallback token color when the theme gives none.
const CODE_FG_FALLBACK: u32 = 0xc0c5ce;

/// Markdown → display blocks, via comrak's AST.
///
/// Parses CommonMark + common GFM extensions (tables, strikethrough, autolinks,
/// task lists) and flattens the tree into [`MdBlock`]s. Raw HTML is never
/// interpreted: it is surfaced as literal text. Links are kept only for
/// `http`/`https`/`mailto` targets. Fenced code is colorized with syntect.
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

    fn options() -> Options<'static> {
        let mut options = Options::default();
        options.extension.table = true;
        options.extension.strikethrough = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options
    }

    fn highlight(&self, lang: Option<&str>, code: &str) -> Vec<Vec<CodeToken>> {
        let syntax = lang
            .and_then(|l| self.syntax_set.find_syntax_by_token(l))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        LinesWithEndings::from(code)
            .map(
                |line| match highlighter.highlight_line(line, &self.syntax_set) {
                    Ok(regions) => regions
                        .into_iter()
                        .map(|(style, text)| CodeToken {
                            text: text.trim_end_matches(['\n', '\r']).to_string(),
                            rgb: rgb(style.foreground),
                        })
                        .filter(|t| !t.text.is_empty())
                        .collect(),
                    // Highlighting is cosmetic: fall back to the plain line.
                    Err(_) => vec![CodeToken {
                        text: line.trim_end_matches(['\n', '\r']).to_string(),
                        rgb: CODE_FG_FALLBACK,
                    }],
                },
            )
            .collect()
    }
}

impl MarkdownRenderer for ComrakMarkdownRenderer {
    fn render_blocks(&self, markdown: &str) -> Vec<MdBlock> {
        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &Self::options());
        let mut walker = Walker {
            renderer: self,
            out: Vec::new(),
        };
        walker.block(root, 0, 0);
        walker.out
    }
}

fn rgb(c: Color) -> u32 {
    (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b)
}

struct Walker<'r> {
    renderer: &'r ComrakMarkdownRenderer,
    out: Vec<MdBlock>,
}

impl Walker<'_> {
    fn push(&mut self, indent: u8, quote_depth: u8, kind: MdBlockKind) {
        self.out.push(MdBlock {
            indent,
            quote_depth,
            kind,
        });
    }

    fn block<'a>(&mut self, node: &'a AstNode<'a>, indent: u8, quote: u8) {
        let value = node.data.borrow().value.clone();
        match value {
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) | NodeValue::Alert(_) => {
                for child in node.children() {
                    self.block(child, indent, quote.saturating_add(1));
                }
            }
            NodeValue::List(list) => {
                for (i, item) in node.children().enumerate() {
                    let marker = match &item.data.borrow().value {
                        NodeValue::TaskItem(t) => ListMarker::Task(t.symbol.is_some()),
                        _ if list.list_type == ListType::Ordered => {
                            ListMarker::Ordered((list.start + i) as u64)
                        }
                        _ => ListMarker::Bullet,
                    };
                    self.item(item, marker, indent, quote);
                }
            }
            NodeValue::Paragraph => {
                let spans = inlines(node);
                if !spans.is_empty() {
                    self.push(indent, quote, MdBlockKind::Paragraph { spans });
                }
            }
            NodeValue::Heading(h) => {
                let spans = inlines(node);
                self.push(
                    indent,
                    quote,
                    MdBlockKind::Heading {
                        level: h.level.clamp(1, 6),
                        spans,
                    },
                );
            }
            NodeValue::ThematicBreak => self.push(indent, quote, MdBlockKind::Rule),
            NodeValue::CodeBlock(cb) => {
                let lang = cb
                    .info
                    .split_whitespace()
                    .next()
                    .map(str::to_string)
                    .filter(|l| !l.is_empty());
                let lines = self.renderer.highlight(lang.as_deref(), &cb.literal);
                let background = self
                    .renderer
                    .theme
                    .settings
                    .background
                    .map(rgb)
                    .unwrap_or(CODE_BG_FALLBACK);
                self.push(
                    indent,
                    quote,
                    MdBlockKind::Code {
                        lang,
                        background,
                        lines,
                    },
                );
            }
            // Raw HTML is shown verbatim, never interpreted.
            NodeValue::HtmlBlock(h) => {
                let text = h.literal.trim_end().to_string();
                if !text.is_empty() {
                    self.push(
                        indent,
                        quote,
                        MdBlockKind::Paragraph {
                            spans: vec![MdSpan {
                                text,
                                ..MdSpan::default()
                            }],
                        },
                    );
                }
            }
            NodeValue::Table(_) => {
                let mut header = Vec::new();
                let mut rows = Vec::new();
                for row in node.children() {
                    let is_header = matches!(row.data.borrow().value, NodeValue::TableRow(true));
                    let cells: Vec<Vec<MdSpan>> = row.children().map(inlines).collect();
                    if is_header {
                        header = cells;
                    } else {
                        rows.push(cells);
                    }
                }
                self.push(indent, quote, MdBlockKind::Table { header, rows });
            }
            // Document, footnotes, description lists… : walk their content.
            _ => {
                for child in node.children() {
                    self.block(child, indent, quote);
                }
            }
        }
    }

    /// A list item: its first paragraph becomes the item text; any further
    /// content (more paragraphs, nested lists, code) is nested one level deeper.
    fn item<'a>(&mut self, item: &'a AstNode<'a>, marker: ListMarker, indent: u8, quote: u8) {
        let mut children = item.children().peekable();
        let spans = match children.peek() {
            Some(first) if matches!(first.data.borrow().value, NodeValue::Paragraph) => {
                let spans = inlines(first);
                children.next();
                spans
            }
            _ => Vec::new(),
        };
        self.push(indent, quote, MdBlockKind::ListItem { marker, spans });
        for child in children {
            self.block(child, indent.saturating_add(1), quote);
        }
    }
}

/// Flatten a node's inline content into styled runs.
fn inlines<'a>(node: &'a AstNode<'a>) -> Vec<MdSpan> {
    let mut out = Vec::new();
    collect_inlines(node, &MdSpan::default(), &mut out);
    out
}

fn collect_inlines<'a>(node: &'a AstNode<'a>, style: &MdSpan, out: &mut Vec<MdSpan>) {
    for child in node.children() {
        let value = child.data.borrow().value.clone();
        match value {
            NodeValue::Text(t) => push_text(out, style, &t),
            NodeValue::SoftBreak => push_text(out, style, " "),
            NodeValue::LineBreak => push_text(out, style, "\n"),
            NodeValue::Code(c) => push_text(
                out,
                &MdSpan {
                    code: true,
                    ..style.clone()
                },
                &c.literal,
            ),
            // Raw inline HTML is shown verbatim, never interpreted.
            NodeValue::HtmlInline(s) | NodeValue::Raw(s) => push_text(out, style, &s),
            NodeValue::Emph => collect_inlines(
                child,
                &MdSpan {
                    italic: true,
                    ..style.clone()
                },
                out,
            ),
            NodeValue::Strong => collect_inlines(
                child,
                &MdSpan {
                    bold: true,
                    ..style.clone()
                },
                out,
            ),
            NodeValue::Strikethrough => collect_inlines(
                child,
                &MdSpan {
                    strike: true,
                    ..style.clone()
                },
                out,
            ),
            NodeValue::Link(l) => collect_inlines(
                child,
                &MdSpan {
                    link: is_safe_link(&l.url).then(|| l.url.trim().to_string()),
                    ..style.clone()
                },
                out,
            ),
            // Images are never fetched: only their alt text is shown.
            NodeValue::Image(_) => collect_inlines(
                child,
                &MdSpan {
                    link: None,
                    ..style.clone()
                },
                out,
            ),
            _ => collect_inlines(child, style, out),
        }
    }
}

/// Append text in `style`, merging with the previous run when the style matches.
fn push_text(out: &mut Vec<MdSpan>, style: &MdSpan, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = out.last_mut() {
        let same = last.bold == style.bold
            && last.italic == style.italic
            && last.strike == style.strike
            && last.code == style.code
            && last.link == style.link;
        if same {
            last.text.push_str(text);
            return;
        }
    }
    out.push(MdSpan {
        text: text.to_string(),
        ..style.clone()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(md: &str) -> Vec<MdBlock> {
        ComrakMarkdownRenderer::new().render_blocks(md)
    }

    fn span(text: &str) -> MdSpan {
        MdSpan {
            text: text.to_string(),
            ..MdSpan::default()
        }
    }

    #[test]
    fn renders_headings_and_emphasis() {
        let blocks = render("# Titre\n\nsome **bold** and *it* text");
        assert_eq!(
            blocks[0].kind,
            MdBlockKind::Heading {
                level: 1,
                spans: vec![span("Titre")]
            }
        );
        let MdBlockKind::Paragraph { spans } = &blocks[1].kind else {
            panic!("paragraph expected: {blocks:?}");
        };
        assert_eq!(spans[0], span("some "));
        assert!(spans[1].bold && spans[1].text == "bold");
        assert!(spans[3].italic && spans[3].text == "it");
    }

    #[test]
    fn raw_html_is_literal_text_never_markup() {
        let blocks = render("hello <script>alert(1)</script> world");
        let MdBlockKind::Paragraph { spans } = &blocks[0].kind else {
            panic!("paragraph expected: {blocks:?}");
        };
        let text: String = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(text, "hello <script>alert(1)</script> world");
        assert!(spans.iter().all(|s| s.link.is_none()));
    }

    #[test]
    fn neutralizes_javascript_links_and_keeps_web_links() {
        let blocks = render("[bad](javascript:alert(1)) [good](https://example.org)");
        let MdBlockKind::Paragraph { spans } = &blocks[0].kind else {
            panic!("paragraph expected: {blocks:?}");
        };
        // The neutralized link merges with the following plain text.
        let bad = spans.iter().find(|s| s.text.starts_with("bad")).unwrap();
        let good = spans.iter().find(|s| s.text == "good").unwrap();
        assert_eq!(bad.link, None);
        assert_eq!(good.link.as_deref(), Some("https://example.org"));
    }

    #[test]
    fn renders_task_and_nested_lists() {
        let blocks = render("- [x] done\n- [ ] todo\n  1. sub\n  2. sub2");
        assert_eq!(
            blocks[0].kind,
            MdBlockKind::ListItem {
                marker: ListMarker::Task(true),
                spans: vec![span("done")]
            }
        );
        assert_eq!(
            blocks[1].kind,
            MdBlockKind::ListItem {
                marker: ListMarker::Task(false),
                spans: vec![span("todo")]
            }
        );
        assert_eq!(blocks[2].indent, 1);
        assert_eq!(
            blocks[3].kind,
            MdBlockKind::ListItem {
                marker: ListMarker::Ordered(2),
                spans: vec![span("sub2")]
            }
        );
    }

    #[test]
    fn highlights_code_blocks() {
        let blocks = render("```rust\nfn main() {}\n```");
        let MdBlockKind::Code {
            lang,
            background,
            lines,
        } = &blocks[0].kind
        else {
            panic!("code expected: {blocks:?}");
        };
        assert_eq!(lang.as_deref(), Some("rust"));
        assert_ne!(*background, 0);
        assert_eq!(lines.len(), 1);
        let text: String = lines[0].iter().map(|t| t.text.as_str()).collect();
        assert_eq!(text, "fn main() {}");
        // Several distinct colors: the tokens really were highlighted.
        let mut colors: Vec<u32> = lines[0].iter().map(|t| t.rgb).collect();
        colors.dedup();
        assert!(colors.len() > 1, "not highlighted: {lines:?}");
    }

    #[test]
    fn quotes_rules_and_tables() {
        let blocks = render("> quoted\n\n---\n\n| a | b |\n|---|---|\n| 1 | 2 |");
        assert_eq!(blocks[0].quote_depth, 1);
        assert_eq!(blocks[1].kind, MdBlockKind::Rule);
        assert_eq!(
            blocks[2].kind,
            MdBlockKind::Table {
                header: vec![vec![span("a")], vec![span("b")]],
                rows: vec![vec![vec![span("1")], vec![span("2")]]],
            }
        );
    }

    #[test]
    fn empty_markdown_yields_no_block() {
        assert!(render("").is_empty());
        assert!(render("   \n\n").is_empty());
    }
}
