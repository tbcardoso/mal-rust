use std::collections::vec_deque::VecDeque;
use std::fmt;
use types::MalError::*;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct MalValue {
    pub mal_type: Rc<MalValueType>,
    // Possible extra fields: line, column
}

impl MalValue {
    pub fn new(mal_type: MalValueType) -> MalValue {
        MalValue { mal_type: Rc::new(mal_type) }
    }
}

#[derive(Debug, PartialEq)]
pub enum MalValueType {
    Number(f64),
    Symbol(String),
    Str(String),
    List(VecDeque<MalValue>),
}

#[derive(Debug, PartialEq)]
pub enum MalError {
    EmptyProgram,
    Tokenizer(String),
    Parser(String),
    UndefinedSymbol(String),
}

impl fmt::Display for MalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmptyProgram => write!(f, "Empty program."),
            Tokenizer(message) => write!(f, "Tokenizer error: {}", message),
            Parser(message) => write!(f, "Parser error: {}", message),
            UndefinedSymbol(symbol) => write!(f, "Undefined symbol: {}", symbol),
        }
    }
}

pub type MalResult = Result<MalValue, MalError>;

#[derive(Debug, PartialEq)]
pub struct MalToken {
    pub token_type: MalTokenType,
    // Possible extra fields: line, column
}

impl MalToken {
    pub fn new(token_type: MalTokenType) -> MalToken {
        MalToken { token_type }
    }
}

#[derive(Debug, PartialEq)]
pub enum MalTokenType {
    LParen,
    RParen,
    LCurly,
    RCurly,
    LBracket,
    RBracket,
    Number(f64),
    Symbol(String),
    Str(String),
}
