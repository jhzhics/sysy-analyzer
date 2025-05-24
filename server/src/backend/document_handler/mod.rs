use std::sync::Arc;
use tower_lsp::lsp_types::{Position, Range, SemanticToken};

mod  semantic;

pub struct DocHandler
{
    tree: Arc<tree_sitter::Tree>,
}

impl DocHandler {
    pub fn new(content: &str, parser: &mut tree_sitter::Parser) -> Self {
        let tree = parser.parse(content, None).expect("Failed to parse document");
        DocHandler {
            tree: Arc::new(tree),
        }
    }
}