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
            let line = format!("{}\n", line);
            line_bytes.push(line.bytes().len());
            content.push(StringWrapper::from(line));
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
        offset += the_line.0.char_indices().take(column).map(|(i, _)| i).sum::<usize>();

        offset
    }

    pub fn apply_change(&mut self, &input_edit: &InputEdit, new_text: &str) {
        let new_lines_count = new_text.lines().count();
        let mut new_lines: Vec<StringWrapper> = new_text
            .lines().enumerate()
            .map(|(idx, line)| {
                if idx != new_lines_count - 1 {
                    StringWrapper::from(format!("{}\n", line))
                } else {
                    StringWrapper::from(line.to_string())
                }
            })
            .collect();
        new_lines[0].0 = self.content.get(input_edit.start_position.row).expect("Start position row should exist")
        .0[..input_edit.start_position.column].to_string() + &new_lines[0].0;
        let back_idx = new_lines.len() - 1;
        new_lines[back_idx].0 = new_lines[back_idx].0.clone()
        + &self.content.get(input_edit.new_end_position.row).expect("New end position row should exist").0[input_edit.new_end_position.column..];

        self.line_bytes.remove_range(input_edit.start_position.row..input_edit.old_end_position.row + 1);
        for line in new_lines.iter() {
            self.line_bytes.insert_after_k_nodes(input_edit.start_position.row, line.0.bytes().len());
        }
        self.content.remove_range(input_edit.start_position.row..input_edit.old_end_position.row + 1);
        for (idx, line) in new_lines.into_iter().enumerate() {
            self.content.insert_after_k_nodes(input_edit.start_position.row + idx, line);
        }
    }

    pub fn get_text(&self, row: usize, column: usize) -> &[u8] {
        let line = self.content.get(row);
        if let Some(line) = line {
            let byte_offset = line.0.char_indices().take(column).map(|(i, _)| i).sum::<usize>();
            &line.0.as_bytes()[byte_offset..]
        } else {
            &[]
        }
    }
}

impl DocHandler {
        pub async fn incremental_update(&mut self, change: &TextDocumentContentChangeEvent, parser: &mut tree_sitter::Parser) {
        let mut doc = self.doc.lock().await;
        let tower_lsp::lsp_types::Range{ start, end} = change.range.expect("Range should be defined");
        let start_byte = doc.get_byte_offset(start.line as usize, start.character as usize);
        let old_end_byte = doc.get_byte_offset(end.line as usize, end.character as usize);
        let new_end_byte = start_byte + change.text.bytes().len();

        let input_edit = InputEdit{
            start_byte,
            old_end_byte,
            new_end_byte,
            start_position: tree_sitter::Point { row: start.line as usize, column: start.character as usize },
            old_end_position: tree_sitter::Point { row: end.line as usize, column: end.character as usize },
            new_end_position: tree_sitter::Point { row: start.line as usize + change.text.lines().count() - 1,
            column: change.text.lines().last().unwrap_or("").chars().count() },
        };
        
        let mut tree = self.tree.lock().await;
        tree.edit(&input_edit);
        doc.apply_change(&input_edit, &change.text);
        let mut get_text_callback = |bytes: usize, position: tree_sitter::Point| {
            doc.get_text(position.row, position.column)
        };
        let new_tree = parser.parse_with_options(&mut get_text_callback, Some(&tree), None);
        drop(tree); // Release the lock before updating the tree
        self.tree = tokio::sync::Mutex::new(new_tree.expect("Failed to parse document"));
    }
}