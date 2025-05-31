use tower_lsp::lsp_types::{Position, Range};
use tree_sitter::Point;

use super::DocHandler;

impl DocHandler {
    pub fn get_token_at_position(&self, pos: Position) -> Option<Range> {
        let start_pos = Point {
            row: pos.line as usize,
            column: pos.character as usize,
        };
        let end_pos = Point {
            row: pos.line as usize,
            column: pos.character as usize,
        };
        let node = self.syntax_tree.root_node().descendant_for_point_range(
            start_pos,
            end_pos,
        )?;
        Some(
            Range {
                start: Position {
                    line: node.start_position().row as u32,
                    character: node.start_position().column as u32,
                },
                end: Position {
                    line: node.end_position().row as u32,
                    character: node.end_position().column as u32,
                },
            }
        )
    }
}