use std::path::Display;
use std::collections::BTreeSet;
use tower_lsp::lsp_types::Position;

use super::DocHandler;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Function,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolKind::Variable => write!(f, "variable"),
            SymbolKind::Function => write!(f, "function"),
        }
    }
}


fn query_symbols(mut n: tree_sitter::Node, get_text_range: &impl Fn(tree_sitter::Point, tree_sitter::Point) -> String) -> Vec<Symbol>
{
    let mut symbols = Vec::new();

    loop {
        if n.prev_named_sibling().is_some() {
        n = n.prev_named_sibling().unwrap();
        } else if n.parent().is_some() {
            n = n.parent().unwrap();
        } else {
            break;
        }
        if n.kind() == "VarDecl" || n.kind() == "ConstDecl" {
            // Check variable/constant definitions
            let mut cursor = n.walk();
            let defs = n.children_by_field_name("defs", &mut cursor);
            for def in defs {
                if let Some(def_ident) = def.child_by_field_name("ident") {
                    let def_name = get_text_range(
                        def_ident.start_position(), def_ident.end_position());
                    symbols.push(Symbol {
                        name: def_name,
                        kind: SymbolKind::Variable,
                    });
                }
            }
        }
        else if n.kind() == "FuncDef"
        {
            let func_name = n.child_by_field_name("ident");
            if func_name.is_none() {
                continue; // Skip if no function name found
            }
            let func_name = func_name.unwrap();
            let name = get_text_range(
            func_name.start_position(), func_name.end_position());
            symbols.push(Symbol {
                name,
                kind: SymbolKind::Function,
            });
            
            // Check function parameters
            let mut cursor = n.walk();
            let params = n.children_by_field_name("params", &mut cursor);
            for param_node in params {
            if let Some(param_ident) = param_node.child_by_field_name("ident") {
                let param_name = get_text_range(
                param_ident.start_position(), param_ident.end_position());
                symbols.push(Symbol {
                    name: param_name,
                    kind: SymbolKind::Variable,
                });
            }
            }
        }

    }
    // Use a BTreeSet to track the unique symbol names we've already seen
    let mut seen_names = BTreeSet::new();
    let mut unique_symbols = Vec::new();

    for symbol in symbols {
        if seen_names.insert(symbol.name.clone()) {
            // Only add symbol if its name wasn't already in the set
            unique_symbols.push(symbol);
        }
    }

    unique_symbols
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub name: String,
}

impl DocHandler {
    pub fn query_symbols(&self, position: Position) -> Vec<Symbol> {
        let node = self.syntax_tree.root_node().descendant_for_point_range(
            tree_sitter::Point {
                row: position.line as usize,
                column: position.character as usize,
            },
            tree_sitter::Point {
                row: position.line as usize,
                column: position.character as usize + 1,
            },
        );

        if let Some(node) = node {
            query_symbols(node, &|start, end| self.doc.get_text_range(start, end))
        } else {
            Vec::new()
        }
    }   
}