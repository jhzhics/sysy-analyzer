use colored::Colorize;

use super::symtable::{*};

pub fn symbol_decl_pass(
    symbol_table: &mut SymbolTable,
    node: &tree_sitter::Node,
    get_text_range: &impl Fn(tree_sitter::Point, tree_sitter::Point) -> String,
)
{
    
    match node.kind() {
        "VarDecl" => {
            let type_name = node.child_by_field_name("type").map_or("".to_string(),
            |n| get_text_range(n.start_position(), n.end_position()));
            let def_nodes = node.children_by_field_name("defs", &mut node.walk()).collect::<Vec<_>>();
            for def_node in def_nodes {
                let ident_name = def_node.child_by_field_name("ident").map_or("".to_string(),
                |n| get_text_range(n.start_position(), n.end_position()));
                let array_qualifier = def_node.child_by_field_name("array_qualifier").map_or("".to_string(),
                |n| get_text_range(n.start_position(), n.end_position()));

                if symbol_table.get(&ident_name).is_some()
                {
                    eprintln!("Error: Variable '{}' is already declared.", ident_name);
                    // TODO: Handle error appropriately(report)
                }
                else
                {
                    symbol_table.insert(
                        ident_name.clone(),
                        Symbol::Variable {
                            type_: format!("{} {}", type_name.blue(), array_qualifier)
                        }
                    );
                }
            }
        },
        "ConstDecl" => {
            let type_name = node.child_by_field_name("type").map_or("".to_string(),
            |n| get_text_range(n.start_position(), n.end_position()));
            let def_nodes = node.children_by_field_name("defs", &mut node.walk()).
            collect::<Vec<_>>();
            for def_node in def_nodes {
                let ident_name = def_node.child_by_field_name("ident").map_or("".to_string(),
                |n| get_text_range(n.start_position(), n.end_position()));
                let array_qualifier = def_node.child_by_field_name("array_qualifier").map_or("".to_string(),
                |n| get_text_range(n.start_position(), n.end_position()));

                if symbol_table.get(&ident_name).is_some()
                {
                    eprintln!("Error: Variable '{}' is already declared.", ident_name);
                    // TODO: Handle error appropriately(report)
                }
                else
                {
                    symbol_table.insert(
                        ident_name.clone(),
                        Symbol::Variable {
                            type_: format!("{} {} {}", "const".blue() ,type_name.blue(), array_qualifier)
                        }
                    );
                }
            }
        },
        "FuncDef" => {
            let return_type = node.child_by_field_name("type").map_or("".to_string(),
            |n| get_text_range(n.start_position(), n.end_position()));
            let func_name = node.child_by_field_name("ident").map_or("".to_string(),
            |n| get_text_range(n.start_position(), n.end_position()));
            let params = node.children_by_field_name("params", &mut node.walk()).
             map(|param_node| {
                println!("Param Node: {}", param_node.kind());
                let type_name = param_node.child_by_field_name("type").map_or("".to_string(),
                |n| get_text_range(n.start_position(), n.end_position()));
                let array_qualifier = param_node.child_by_field_name("array_qualifier").map_or("".to_string(),
                |n| get_text_range(n.start_position(), n.end_position()));
                Symbol::Variable { type_:
                    format!("{} {}", type_name.blue(), array_qualifier) }
                }
            ).collect::<Vec<_>>();
            if symbol_table.get(&func_name).is_some()
            {
                eprintln!("Error: Function '{}' is already declared.", func_name);
            }
            else
            {
                symbol_table.insert(
                    func_name.clone(),
                    Symbol::Function {
                        return_type: return_type.blue().to_string(),
                        params
                    }
                );
            }
        },
        _ => {}
    }
}