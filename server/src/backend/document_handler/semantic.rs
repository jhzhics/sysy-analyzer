use tower_lsp::lsp_types::{SemanticToken};

use super::{incremental_update::DynText, DocHandler};

impl DocHandler {
    pub async fn get_semantic_tokens(&self) -> Vec<SemanticToken> {
        let mut result:Vec<SemanticToken> = Vec::new();
        let mut prev_line = 0;
        let mut prev_start = 0;
        let tree = self.tree.lock().await;
        let doc = self.doc.lock().await;
        // Traverse tree in pre-order
        self.traverse_tree(
            &tree.root_node(), 
            &mut result, 
            &mut prev_line, 
            &mut prev_start,
            &doc
        );
        
        result
    }
    
    fn traverse_tree(
        &self,
        node: &tree_sitter::Node,
        tokens: &mut Vec<SemanticToken>,
        prev_line: &mut u32,
        prev_start: &mut u32,
        dyn_text: &DynText,
    ) {
        // Process this node if it matches a token type
    let token_type = match node.kind() {
        // Keywords
        "if" | "else" | "while" | "break" |  "continue" | "return" | "const" => 0, // KEYWORD
    
        // Variables and identifiers
        "Ident" => {
            // Check parent to differentiate between variables and functions
            let parent = node.parent();
            if let Some(p) = parent {
                if p.kind() == "FuncDef" && p.child(1) == Some(*node) {
                    1 // FUNCTION (name in declaration)
                } else if p.kind() == "FuncCall" && p.child(0) == Some(*node) {
                    1 // FUNCTION (in call)
                } else {
                    2 // VARIABLE
                }
            } else {
                2 // VARIABLE (default)
            }
        }
        
        // Numbers
        "Number" => 3, // NUMBER
        
        // Comments
        "comment" => 4, // COMMENT
        
        // Operators
        "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | 
        "<=" | ">=" | "&&" | "||" | "!" => 5, // OPERATOR
        
        // Types
        "Type" => 6, // TYPE
        
        // All others
        _ => 255, // Not a token we're interested in
    };
        
        if token_type != 255 {
            let start = node.start_position();
            let end = node.end_position();
            
            // Calculate deltas from previous token
            let delta_line = start.row as u32 - *prev_line;
            let delta_start = if delta_line == 0 {
                start.column as u32 - *prev_start
            } else {
                start.column as u32
            };
            if start.row == end.row {
                tokens.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: (end.column as u32 - start.column as u32) as u32,
                    token_type: token_type,
                    token_modifiers_bitset: 0, // No token modifiers
                });
                // Update previous position
                *prev_line = start.row as u32;
                *prev_start = start.column as u32;
            }
            else
            {
                // Push First Line Token
                let the_line = dyn_text.get_line(start.row as usize).expect("Failed to get line");
                let first_line_length = the_line.len() as u32 - start.column as u32;
                tokens.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: first_line_length,
                    token_type: token_type,
                    token_modifiers_bitset: 0, // No token modifiers
                });
                for line in start.row as usize + 1..end.row as usize {
                    let the_line = dyn_text.get_line(line).expect("Failed to get line");
                    tokens.push(SemanticToken {
                        delta_line: 1, // Each subsequent line is a new line
                        delta_start: 0, // Start at beginning of line
                        length: the_line.len() as u32,
                        token_type: token_type,
                        token_modifiers_bitset: 0, // No token modifiers
                    });
                }
                // Push Last Line Token
                let last_line_length = end.column as u32;
                tokens.push(SemanticToken {
                    delta_line: 1, // Last line is a new line
                    delta_start: 0, // Start at beginning of line
                    length: last_line_length,
                    token_type: token_type,
                    token_modifiers_bitset: 0, // No token modifiers
                });
                // Update previous position
                *prev_line = end.row as u32;
                *prev_start = 0;
            }
        }
        
        // Process all children
        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i) {
                self.traverse_tree(&child, tokens, prev_line, prev_start, dyn_text);
            }
        }
    }

    
}