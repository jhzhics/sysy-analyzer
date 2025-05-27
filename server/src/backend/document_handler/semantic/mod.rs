use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::collections::{LinkedList};
use tree_sitter::{InputEdit, TreeCursor};
mod semantic_token;
mod semantic_pass;
mod symtable;

#[derive(Debug)]
pub struct SemanticModel {
    root: Node
}

struct BlockAttr{
    pub sybol_table: symtable::SymbolTable,
}

#[derive(Debug)]
enum Node {
    Block{
        children: LinkedList<Box<Node>>,
        length: AtomicUsize,
        dirty: AtomicBool,
    },
    PlaceHolder {
        length: usize
    }
}

impl Node {
    fn get_length(&self) -> usize {
        match self {
            Node::Block { length, dirty: lazy, ..} => {
                if lazy.load(std::sync::atomic::Ordering::Relaxed) {
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
        if let Node::Block { children, length, dirty: lazy} = self {
            if lazy.swap(false, std::sync::atomic::Ordering::Relaxed) {
                let sum_length: usize = children.iter().map(|child| child.get_length()).sum();
                length.store(sum_length as usize, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    fn set_dirty(&self) {
        if let Node::Block { dirty: lazy, .. } = self {
            lazy.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    fn new(cursor: &mut TreeCursor) -> Self {
        if cursor.node().kind() != "Block" && cursor.node().kind() != "source_file" {
            panic!("Cursor must be at a Block or source_file node, found: {}", cursor.node().kind());
        }

        let mut children = LinkedList::new();
        let mut prev_block_bound = cursor.node().start_byte();
        let mut block_attr = BlockAttr {
            sybol_table: symtable::SymbolTable::new(),
        };
        if cursor.goto_first_child() {
            Self::from_cursor(cursor, &mut children, &mut prev_block_bound, &mut block_attr);
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
        }
    }


    fn from_cursor(cursor: &mut TreeCursor, children: &mut LinkedList<Box<Node>>, prev_block_bound: &mut usize,
        block_attr: &mut BlockAttr)
    {
        if cursor.node().kind() == "Block" || cursor.node().kind() == "source_file" {
            if cursor.node().start_byte() - *prev_block_bound > 0 {
                children.push_back(Box::new(Node::PlaceHolder { length: cursor.node().start_byte() - *prev_block_bound }));
                *prev_block_bound = cursor.node().end_byte();
            }
            children.push_back(Box::new(Node::new(cursor)));
        }
        else if cursor.goto_first_child() {
            Self::from_cursor(cursor, children, prev_block_bound, block_attr);
            cursor.goto_parent();
        }

        if cursor.goto_next_sibling() {
            Self::from_cursor(cursor, children, prev_block_bound, block_attr);
        }
    }


    
}



impl SemanticModel {
    pub fn new(mut root_cursor: TreeCursor) -> Self {
        assert!(root_cursor.node().kind() == "source_file",
            "Root cursor must be at a source_file node, found: {}", root_cursor.node().kind());

        let mut children = LinkedList::new();
        if root_cursor.node().start_byte() > 0 {
            children.push_back(Box::new(Node::PlaceHolder { length: root_cursor.node().start_byte() }));
        }
        children.push_back(Box::new(Node::new(&mut root_cursor)));
        let root = Node::Block {
            children,
            length: AtomicUsize::new(0),
            dirty: AtomicBool::new(true),
        };
        SemanticModel { root }    
    }

    pub fn incremental_update(&mut self, edit: InputEdit, new_tree: &tree_sitter::Tree) {
        
    }
}

