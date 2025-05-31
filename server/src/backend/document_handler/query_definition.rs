
use super::DocHandler;
use tower_lsp::lsp_types::{Position, Range};
use tree_sitter::{Node, Point};

fn find_definition<'a>(ident: &'a str, mut n: tree_sitter::Node<'a>, get_text_range: &'a impl Fn(tree_sitter::Point, tree_sitter::Point) -> String)
-> Option<Node<'a>>
{
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum LastJump
    {
        FromSibling,
        FromChild,
        NoJump,
    }
    let mut last_jump = LastJump::NoJump;
    loop {
        if n.prev_named_sibling().is_some() {
            n = n.prev_named_sibling().unwrap();
            last_jump = LastJump::FromSibling;
        } else if n.parent().is_some() {
            n = n.parent().unwrap();
            last_jump = LastJump::FromChild;
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
                    if def_name == ident {
                        return Some(n);
                    }
                }
            }
        }
        else if n.kind() == "FuncDef"
        {
            let func_name = n.child_by_field_name("ident")?;
            let name = get_text_range(
            func_name.start_position(), func_name.end_position());
            if name == ident {
            // Found a function with matching name
            return Some(n);
            }
            
            if last_jump == LastJump::FromSibling
            {
                continue;
            }
        
            // Check function parameters
            let mut cursor = n.walk();
            let params = n.children_by_field_name("params", &mut cursor);
            for param_node in params {
            if let Some(param_ident) = param_node.child_by_field_name("ident") {
                let param_name = get_text_range(
                param_ident.start_position(), param_ident.end_position());
                if param_name == ident {
                return Some(param_node);
                }
            }
            }
        }
    }
    None
}

impl DocHandler {
    pub fn find_definition(&self, pos: Position) -> Option<Range>
    {
        let node = self.syntax_tree.root_node().descendant_for_point_range(
            Point {
                row: pos.line as usize,
                column: pos.character as usize,
            },
            Point {
                row: pos.line as usize,
                column: pos.character as usize + 1,
            },
        )?;
        if node.kind() != "Ident"
        {
            return None;
        }
        let name = self.doc.get_text_range(node.start_position(), node.end_position());
        let get_text = |start: Point, end: Point| self.doc.get_text_range(start, end);
        let definition = {
            find_definition(name.as_str(), node, &get_text)
        }?;
        Some(
            Range {
                start: Position {
                    line: definition.start_position().row as u32,
                    character: definition.start_position().column as u32,
                },
                end: Position {
                    line: definition.end_position().row as u32,
                    character: definition.end_position().column as u32,
                },
            }
        )
    }
}