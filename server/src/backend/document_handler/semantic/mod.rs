use std::fmt::{Debug, Pointer};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::collections::{LinkedList};
use tower_lsp::lsp_types::Position;
use tree_sitter::{InputEdit, Point, TreeCursor};
mod semantic_pass;
mod symtable;

#[derive(Debug)]
pub struct SemanticModel {
    root: Node
}

struct BlockAttr{
    pub symbol_table: symtable::SymbolTable,
}

impl Debug for BlockAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockAttr {{ symbol_table: {{")?;
        let mut first = true;
        for (key, value) in &self.symbol_table {
            if first {
            first = false;
            } else {
            write!(f, ", ")?;
            }
            write!(f, "{}: {:?}", key, value)?;
        }
        write!(f, "}} }}")
    }
    
}

#[derive(Debug)]
enum Node {
    Block{
        children: LinkedList<Box<Node>>,
        length: AtomicUsize,
        dirty: AtomicBool,
        attr: BlockAttr
    },
    PlaceHolder {
        length: usize
    }
}

impl Node {
    fn get_length(&self) -> usize {
        match self {
            Node::Block { length, dirty, ..} => {
                if dirty.load(std::sync::atomic::Ordering::Relaxed) {
                    self.update();
                }
                length.load(std::sync::atomic::Ordering::Relaxed) as usize
            }
            Node::PlaceHolder { length } => {
                *length as usize
            }
        }
    }

    fn update(&self) {
        if let Node::Block { children, length, dirty, ..} = self {
            if dirty.swap(false, std::sync::atomic::Ordering::Relaxed) {
                let sum_length: usize = children.iter().map(|child| child.get_length()).sum();
                length.store(sum_length as usize, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    fn set_dirty(&self) {
        if let Node::Block { dirty, .. } = self {
            dirty.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    fn new(cursor: &mut TreeCursor, get_text_range: &impl Fn(Point, Point) -> String) -> Self {
        if cursor.node().kind() != "Block" && cursor.node().kind() != "source_file" {
            panic!("Cursor must be at a Block or source_file node, found: {}", cursor.node().kind());
        }

        let mut children = LinkedList::new();
        let mut prev_block_bound = cursor.node().start_byte();
        let mut block_attr = BlockAttr {
            symbol_table: symtable::SymbolTable::new(),
        };
        if cursor.goto_first_child() {
            Self::from_cursor(cursor, &mut children, &mut prev_block_bound, &mut block_attr, get_text_range);
            cursor.goto_parent();
            if cursor.node().end_byte() - prev_block_bound > 0 {
                children.push_back(Box::new(Node::PlaceHolder { length: cursor.node().end_byte() - prev_block_bound }));
            }
        }
        else {
            children.push_back(Box::new(Node::PlaceHolder { length: cursor.node().end_byte() - cursor.node().start_byte() }));
        }
        debug_assert_eq!(children.iter().map(|child| child.get_length()).sum::<usize>(),
        cursor.node().end_byte() - cursor.node().start_byte() as usize,
        "Children lengths do not match node length");
        Node::Block {
            children,
            length: AtomicUsize::new(cursor.node().end_byte() - cursor.node().start_byte()),
            dirty: AtomicBool::new(false),
            attr: block_attr,
        }
    }


    fn from_cursor(cursor: &mut TreeCursor, children: &mut LinkedList<Box<Node>>, prev_block_bound: &mut usize,
        block_attr: &mut BlockAttr, get_text_range: &impl Fn(Point, Point) -> String)
    {
        semantic_pass::symbol_decl_pass(&mut block_attr.symbol_table, &cursor.node(), get_text_range);

        if cursor.node().kind() == "Block" || cursor.node().kind() == "source_file" {
            if cursor.node().start_byte() - *prev_block_bound > 0 {
                children.push_back(Box::new(Node::PlaceHolder { length: cursor.node().start_byte() - *prev_block_bound }));
                *prev_block_bound = cursor.node().end_byte();
            }
            children.push_back(Box::new(Node::new(cursor, get_text_range)));
        }
        else if cursor.goto_first_child() {
            Self::from_cursor(cursor, children, prev_block_bound, block_attr, get_text_range);
            cursor.goto_parent();
        }

        if cursor.goto_next_sibling() {
            Self::from_cursor(cursor, children, prev_block_bound, block_attr, get_text_range);
        }
    }

    pub fn text_edit(&mut self, start_byte: usize, old_size: usize, new_size: usize)
    {
        if (self.get_length() < start_byte + old_size) {
            panic!("Text edit out of bounds: start_byte: {}, old_size: {}, new_size: {}, node length: {}", 
                start_byte, old_size, new_size, self.get_length());
        }
        match self {
            Node::Block { children, length, .. } => {
            },
            Node::PlaceHolder { length } => {
                *length = *length - old_size + new_size;
            }
        }
    }
    
}



impl SemanticModel {
    pub fn new(mut root_cursor: TreeCursor, get_text_range: &impl Fn(Point, Point) -> String) -> Self {
        assert!(root_cursor.node().kind() == "source_file",
            "Root cursor must be at a source_file node, found: {}", root_cursor.node().kind());

        let mut children = LinkedList::new();
        if root_cursor.node().start_byte() > 0 {
            children.push_back(Box::new(Node::PlaceHolder { length: root_cursor.node().start_byte() }));
        }
        children.push_back(Box::new(Node::new(&mut root_cursor, get_text_range)));
        let root = Node::Block {
            children,
            length: AtomicUsize::new(0),
            dirty: AtomicBool::new(true),
            attr: BlockAttr {
                symbol_table: symtable::SymbolTable::new(),
            }
        };
        SemanticModel { root }    
    }

    pub fn incremental_update(&mut self, edit: InputEdit, new_tree: &tree_sitter::Tree) {
        
    }
}

