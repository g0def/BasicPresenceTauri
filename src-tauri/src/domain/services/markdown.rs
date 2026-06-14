/// Port for rendering Markdown into sanitized, display-safe HTML.
///
/// The backend is the single Markdown→HTML converter so the WebView never has to
/// render untrusted Markdown itself; implementations MUST sanitize their output.
pub trait MarkdownRenderer: Send + Sync {
    fn render_to_safe_html(&self, markdown: &str) -> String;
}
