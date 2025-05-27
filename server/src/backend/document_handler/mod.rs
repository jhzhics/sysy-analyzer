use tokio::sync::Mutex;

mod semantic;
mod incremental_update;

pub struct DocHandler
{
    syntax_tree: Mutex<tree_sitter::Tree>,
    semantic_tree: Mutex<semantic::SemanticModel>,
    doc: Mutex<incremental_update::DynText>,
}


impl DocHandler {
    pub fn new(content: &str, parser: &mut tree_sitter::Parser) -> Self {
        let tree = parser.parse(content, None).expect("Failed to parse document");
        let semantic_tree = semantic::SemanticModel::new(tree.walk());
        let doc = incremental_update::DynText::new(content);
        println!("Semanticl Model: {:?}", semantic_tree);
        DocHandler {
            semantic_tree: Mutex::new(semantic_tree),
            syntax_tree: Mutex::new(tree),
            doc: Mutex::new(doc),
        }
    }
}