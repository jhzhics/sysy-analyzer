use std::collections::HashMap;
pub enum Symbol {
    Variable {
        type_: String,
    },
    Function {
        return_type: String,
        parameters: Vec<Symbol>,
    },
}

pub type SymbolTable = HashMap<String, Symbol>;