use std::{ops::{Add, Deref}};

use super::DocHandler;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent};
use tree_sitter::{InputEdit, Point};
use treaplist::TreapList;

#[derive(Debug, Clone, Default)]
struct StringWrapper(String);

impl Add<StringWrapper> for StringWrapper {
    type Output = StringWrapper;

    fn add(self, other: StringWrapper) -> Self::Output {
        StringWrapper(format!("{}{}", self.0, other.0))
    }
}

impl From<String> for StringWrapper {
    fn from(s: String) -> Self {
        StringWrapper(s)
    }
}

impl From<StringWrapper> for String {
    fn from(wrapper: StringWrapper) -> Self {
        wrapper.0
    }
}

impl Deref for StringWrapper {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct DynText
{
    content: TreapList<StringWrapper>,
    line_bytes: TreapList<usize>,
}

impl DynText {
    pub fn new(text: &str) -> Self {
        let mut content = TreapList::new();
        let mut line_bytes = TreapList::new();

        text.lines().for_each(|line| {
            let line_length = line.bytes().len();
            content.push(StringWrapper::from(format!("{}\n", line)));
            line_bytes.push(line_length + 1);
        });

        content.push(StringWrapper::from("\n".to_string()));
        line_bytes.push(1);

        DynText {
            content,
            line_bytes,
        }
    }

    pub fn get_byte_offset(&self, line: usize, column: usize) -> usize {
        assert!(line < self.line_bytes.len(), "Line index out of bounds");
        let mut offset = self.line_bytes.sum_range(0..line);
        let the_line = self.content.get(line).expect("Line should exist");
        offset += the_line.0.char_indices().nth(column).expect("Column index out of bounds").0;

        offset
    }

    pub fn apply_change(&mut self, &input_edit: &InputEdit, new_text: &str) {
        let Some(first_line) = self.content.get(input_edit.start_position.row) else {
            panic!("Start position row out of bounds");
        };
        let prefix = if let Some((idx, _)) = first_line.0.char_indices().nth(input_edit.start_position.column) {
            first_line.0[..idx].to_string()
        } else {
            panic!("Start position column out of bounds");
        };
        let Some(last_line) = self.content.get(input_edit.old_end_position.row) else {
            panic!("Old end position row out of bounds");
        };
        let subfix = if let Some((idx, _)) = last_line.0.char_indices().nth(input_edit.old_end_position.column) {
            last_line.0[idx..].to_string()
        } else {
            panic!("Old end position column out of bounds");
        };

        let new_next = format!("{}{}{}", prefix, new_text, subfix);
        self.line_bytes.remove_range(input_edit.start_position.row..input_edit.old_end_position.row + 1);
        for (idx, line) in new_next.lines().enumerate() {
            self.line_bytes.insert_after_k_nodes(input_edit.start_position.row + idx, line.bytes().len() + 1);
        }
        self.content.remove_range(input_edit.start_position.row..input_edit.old_end_position.row + 1);
        for (idx, line) in new_next.lines().enumerate() {
            self.content.insert_after_k_nodes(input_edit.start_position.row + idx, StringWrapper::from(
            format!("{}\n", line)));
        }

    }

    pub fn get_text(&self, row: usize, column: usize) -> &[u8] {
        let line = self.content.get(row);
        if let Some(line) = line {
            let byte_offset = line.0.char_indices().nth(column);
            if let Some((byte_offset, _)) = byte_offset {
                &line.0.as_bytes()[byte_offset..]
            }
            else
            {
                &[]
            }
        } else {
            &[]
        }
    }

    pub fn get_text_range(&self, start: Point, end: Point) -> String {
        assert!(start.row < end.row || (start.row == end.row && start.column <= end.column), "Invalid range");
        if start.row == end.row {
            static EMPTY_STRING_WRAPPER: StringWrapper = StringWrapper(String::new());
            let line = self.content.get(start.row).unwrap_or(&EMPTY_STRING_WRAPPER);
            let start_byte = line.0.char_indices().nth(start.column).unwrap_or((0, '\0')).0;
            let end_byte = line.0.char_indices().nth(end.column).unwrap_or((line.0.len(), '\0')).0;
            return line.0[start_byte..end_byte].to_string();
        }
        else {
            let mut result = String::new();
            // Get the first line
            if let Some(first_line) = self.content.get(start.row) {
                let start_byte = first_line.0.char_indices().nth(start.column).expect("Start column out of bounds").0;
                result.push_str(&first_line.0[start_byte..]);
            }
            // Get the lines in between
            let middle_lines = self.content.sum_range(start.row + 1..end.row);
            result.push_str(&middle_lines.0);
            // Get the last line
            if let Some(last_line) = self.content.get(end.row) {
                let end_byte = last_line.0.char_indices().nth(end.column).expect("End column out of bounds").0;
                result.push_str(&last_line.0[..end_byte]);
            }
            result
        }
    
    
    }
}

impl DocHandler {
        pub async fn incremental_update(&mut self, change: &TextDocumentContentChangeEvent, parser: &mut tree_sitter::Parser) {
        let tower_lsp::lsp_types::Range{ start, end} = change.range.expect("Range should be defined");
        let start_byte = self.doc.get_byte_offset(start.line as usize, start.character as usize);
        let old_end_byte = self.doc.get_byte_offset(end.line as usize, end.character as usize);
        let new_end_byte = start_byte + change.text.bytes().len();
        let text_lf_count = change.text.chars().filter(|&c| c == '\n').count();
        let new_end_row = start.line as usize + text_lf_count;
        let new_end_column = if text_lf_count > 0 {
            change.text.lines().last().unwrap_or("").chars().count()
        } else {
            start.character as usize + change.text.chars().count()
        };

        let input_edit = InputEdit{
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position: tree_sitter::Point { row: start.line as usize, column: start.character as usize },
            old_end_position: tree_sitter::Point { row: end.line as usize, column: end.character as usize },
            new_end_position: tree_sitter::Point { row: new_end_row,              
            column: new_end_column },
        };

        self.syntax_tree.edit(&input_edit);
        self.doc.apply_change(&input_edit, &change.text);
        let mut get_text_callback = |_: usize, position: tree_sitter::Point| {
            self.doc.get_text(position.row, position.column)
        };
        let new_tree = parser.parse_with_options(&mut get_text_callback, Some(&self.syntax_tree), None).
        expect("Failed to parse document");

        // Print the root and all children
        println!("Input Edit Range: {} {}", input_edit.start_byte, input_edit.old_end_byte);
        for change in new_tree.changed_ranges(&self.syntax_tree) {
            println!("Changed range: {:?}", change);
        }
        
        self.syntax_tree = new_tree;
    }

    pub fn get_text_range(&self, start: Point, end: Point) -> String {
        self.doc.get_text_range(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl DynText {
        pub fn sanity_check(&self, text: &str) {
            for (line_idx, line) in text.lines().enumerate() {
                let line_length = line.bytes().len();
                assert_eq!(*self.line_bytes.get(line_idx).unwrap(), line_length + 1);
                assert_eq!(self.content.get(line_idx).unwrap().0, format!("{}\n", line));
            }
            assert_eq!(self.content.len(), text.lines().count());
            assert_eq!(self.line_bytes.len(), text.lines().count());
            assert_eq!(self.content.sum_range(0..self.content.len()).0.trim_end(), text.trim_end());
        }
    }

    #[test]
    fn test_dyn_text1() {
        let text = "Hello\nWorld\nThis is a test\n";
        let mut dyn_text = DynText::new(text);
        dyn_text.sanity_check(text);

        // Test get_byte_offset
        assert_eq!(dyn_text.get_byte_offset(0, 0), 0);
        assert_eq!(dyn_text.get_byte_offset(1, 2), 8);
        assert_eq!(dyn_text.get_byte_offset(2, 5), 17);

        // Test apply_change
        let input_edit = InputEdit {
            start_byte: 6,
            old_end_byte: 11,
            new_end_byte: 12,
            start_position: tree_sitter::Point { row: 1, column: 0 },
            old_end_position: tree_sitter::Point { row: 1, column: 5 },
            new_end_position: tree_sitter::Point { row: 1, column: 6 },
        };
        dyn_text.apply_change(&input_edit, "Universe");
        dyn_text.sanity_check("Hello\nUniverse\nThis is a test\n");
    }
}