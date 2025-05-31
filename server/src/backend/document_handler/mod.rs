mod incremental_update;
mod query_definition;
mod query_token;
pub struct DocHandler
{
    syntax_tree: tree_sitter::Tree,
    doc: incremental_update::DynText,
}


impl DocHandler {
    pub fn new(content: &str, parser: &mut tree_sitter::Parser) -> Self {
        let tree = parser.parse(content, None).expect("Failed to parse document");
        let doc = incremental_update::DynText::new(content);
        DocHandler {
            syntax_tree: tree,
            doc: doc,
        }
    }
}