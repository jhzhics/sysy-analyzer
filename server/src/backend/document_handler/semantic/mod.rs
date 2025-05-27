use tree_sitter::TreeCursor;
mod semantic_token;
struct SemanticModel {
    root: Node
}

enum Node {
    Block{
        next_sibling: Option<Box<Node>>,
        first_child: Option<Box<Node>>,
        length: usize,
    },
    PlaceHolder {
        length: usize,
    }
}

impl Node {
    fn from_cursor(&mut self, cursor: &mut TreeCursor) {
        let mut next_sibling: Option<Box<Node>> = None;
        let mut first_child: Option<Box<Node>> = None;
        let mut length = cursor.node().range().end_byte - cursor.node().range().start_byte;

    }
    
}



impl SemanticModel {
    pub fn new(root_cursor: TreeCursor) -> Self {
        let root = Node::Block {
            next_sibling: None,
            first_child: None,
            length: 0,
        };
        SemanticModel { root }

    
    }
    
}

