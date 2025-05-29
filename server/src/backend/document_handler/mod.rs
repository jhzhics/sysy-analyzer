mod semantic;
mod incremental_update;

pub struct DocHandler
{
    syntax_tree: tree_sitter::Tree,
    semantic_tree: semantic::SemanticModel,
    doc: incremental_update::DynText,
}


impl DocHandler {
    pub fn new(content: &str, parser: &mut tree_sitter::Parser) -> Self {
        let tree = parser.parse(content, None).expect("Failed to parse document");
        let doc = incremental_update::DynText::new(content);
        let semantic_tree = semantic::SemanticModel::new(tree.walk(), 
        &|start, end| doc.get_text_range(start, end));
        println!("Semanticl Model: {:?}", semantic_tree);
        DocHandler {
            semantic_tree: semantic_tree,
            syntax_tree: tree,
            doc: doc,
        }
    }
}