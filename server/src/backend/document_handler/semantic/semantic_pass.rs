use super::symtable::SymbolTable;

pub fn symbol_decl_pass(
    symbol_table: &mut SymbolTable,
    node: &tree_sitter::Node,
    get_text_range: impl Fn(tree_sitter::Point, tree_sitter::Point) -> String,
)
{
    let mut cursor = node.walk();
    
    match node.kind() {
        "VarDecl" => {

        },
        "ConstDecl" => {

        },
        "FuncDef" => {

        }
    }
}