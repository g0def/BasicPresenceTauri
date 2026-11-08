use std::sync::Arc;

use crate::domain::services::markdown::MarkdownRenderer;

/// Renders the app licence (Markdown) into sanitized, display-safe HTML.
///
/// The licence text is static (embedded at compile time by the composition
/// root), so it is rendered exactly once at construction and the resulting HTML
/// is simply handed out on each call.
pub struct GetLicenceUseCase {
    html: String,
}

impl GetLicenceUseCase {
    pub fn new(markdown_src: &str, renderer: Arc<dyn MarkdownRenderer>) -> Self {
        Self {
            html: renderer.render_to_safe_html(markdown_src),
        }
    }

    pub fn execute(&self) -> String {
        self.html.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::markdown::comrak_renderer::ComrakMarkdownRenderer;

    #[test]
    fn renders_embedded_licence_to_html() {
        let renderer: Arc<dyn MarkdownRenderer> = Arc::new(ComrakMarkdownRenderer::new());
        let use_case = GetLicenceUseCase::new("# Basic Presence\n\nsome **text**", renderer);
        let html = use_case.execute();
        assert!(html.contains("<h1>Basic Presence</h1>"), "got: {html}");
        assert!(html.contains("<strong>text</strong>"), "got: {html}");
    }

    #[test]
    fn sanitizes_embedded_html() {
        let renderer: Arc<dyn MarkdownRenderer> = Arc::new(ComrakMarkdownRenderer::new());
        let use_case = GetLicenceUseCase::new("hi <script>alert(1)</script>", renderer);
        assert!(!use_case.execute().contains("<script"));
    }
}
