use std::sync::Arc;

use crate::domain::services::markdown::{MarkdownRenderer, MdBlock};

/// Renders the app licence (Markdown) into display blocks.
///
/// The licence text is static (embedded at compile time by the composition
/// root), so it is rendered exactly once at construction and the resulting
/// blocks are simply handed out on each call.
pub struct GetLicenceUseCase {
    blocks: Vec<MdBlock>,
}

impl GetLicenceUseCase {
    pub fn new(markdown_src: &str, renderer: Arc<dyn MarkdownRenderer>) -> Self {
        Self {
            blocks: renderer.render_blocks(markdown_src),
        }
    }

    pub fn execute(&self) -> Vec<MdBlock> {
        self.blocks.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::services::markdown::{MdBlockKind, MdSpan};
    use crate::infrastructure::markdown::comrak_renderer::ComrakMarkdownRenderer;

    #[test]
    fn renders_embedded_licence_to_blocks() {
        let renderer: Arc<dyn MarkdownRenderer> = Arc::new(ComrakMarkdownRenderer::new());
        let use_case = GetLicenceUseCase::new("# Basic Presence\n\nsome **text**", renderer);
        let blocks = use_case.execute();
        assert_eq!(
            blocks[0].kind,
            MdBlockKind::Heading {
                level: 1,
                spans: vec![MdSpan {
                    text: "Basic Presence".into(),
                    ..MdSpan::default()
                }]
            }
        );
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn real_licence_file_renders() {
        let renderer: Arc<dyn MarkdownRenderer> = Arc::new(ComrakMarkdownRenderer::new());
        let use_case = GetLicenceUseCase::new(include_str!("../../../../../LICENCE"), renderer);
        assert!(!use_case.execute().is_empty());
    }
}
