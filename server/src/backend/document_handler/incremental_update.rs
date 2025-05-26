use std::ops::{Add, Deref};

use super::DocHandler;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent};
use tree_sitter::InputEdit;
use crate::treap_list::TreapList;

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
        println!("New next: {}", new_next);
        let new_next_lines = new_next.lines().collect::<Vec<_>>();
        new_next_lines.iter().for_each(|line| {
            println!("Line: {}", line);
        });
        self.line_bytes.remove_range(input_edit.start_position.row..input_edit.old_end_position.row + 1);
        for line in new_next.lines() {
            self.line_bytes.insert_after_k_nodes(input_edit.start_position.row, line.bytes().len() + 1);
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
            let byte_offset = line.0.char_indices().nth(column).expect("Column index out of bounds").0;
            &line.0.as_bytes()[byte_offset..]
        } else {
            &[]
        }
    }

    pub fn get_line(&self, row: usize) -> Option<&str> {
        self.content.get(row).map(|s| s.0.as_str())
    }
}

impl DocHandler {
        pub async fn incremental_update(&mut self, change: &TextDocumentContentChangeEvent, parser: &mut tree_sitter::Parser) {
        let mut doc = self.doc.lock().await;
        let tower_lsp::lsp_types::Range{ start, end} = change.range.expect("Range should be defined");
        let start_byte = doc.get_byte_offset(start.line as usize, start.character as usize);
        let old_end_byte = doc.get_byte_offset(end.line as usize, end.character as usize);
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
        
        let mut tree = self.tree.lock().await;
        tree.edit(&input_edit);
        doc.apply_change(&input_edit, &change.text);
        let mut get_text_callback = |_: usize, position: tree_sitter::Point| {
            doc.get_text(position.row, position.column)
        };
        let new_tree = parser.parse_with_options(&mut get_text_callback, Some(&tree), None);
        drop(tree); // Release the lock before updating the tree
        self.tree = tokio::sync::Mutex::new(new_tree.expect("Failed to parse document"));
    }
}