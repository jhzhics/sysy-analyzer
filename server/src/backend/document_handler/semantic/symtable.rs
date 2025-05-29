use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable {
        type_: String,
    },
    Function {
        return_type: String,
        params: Vec<Symbol>,
    },
}

pub type SymbolTable = HashMap<String, Symbol>;