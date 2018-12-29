use crate::types::MalError::*;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct MalValue {
    pub mal_type: Rc<MalValueType>,
    // Possible extra fields: line, column
}

impl MalValue {
    pub fn new(mal_type: MalValueType) -> MalValue {
        MalValue {
            mal_type: Rc::new(mal_type),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MalValueType {
    Nil,
    True,
    False,
    Number(f64),
    Symbol(String),
    Str(String),
    List(Vec<MalValue>),
    Vector(Vec<MalValue>),
    RustFunc(RustFunction),
}

pub struct RustFunction(pub fn(&[MalValue]) -> MalResult);

impl fmt::Debug for RustFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("RustFunction")
            .field(&(self.0 as usize))
            .finish()
    }
}

impl PartialEq for RustFunction {
    fn eq(&self, other: &RustFunction) -> bool {
        self.0 as usize == other.0 as usize
    }
}

#[derive(Debug, PartialEq)]
pub enum MalError {
    EmptyProgram,
    Tokenizer(String),
    Parser(String),
    UndefinedSymbol(String),
    Evaluation(String),
    RustFunction(String),
    SpecialForm(String),
}

impl fmt::Display for MalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmptyProgram => write!(f, "Empty program."),
            Tokenizer(message) => write!(f, "Tokenizer error: {}", message),
            Parser(message) => write!(f, "Parser error: {}", message),
            UndefinedSymbol(symbol) => write!(f, "Undefined symbol: {}", symbol),
            Evaluation(message) => write!(f, "Error in evaluation: {}", message),
            MalError::RustFunction(message) => {
                write!(f, "Error when calling rust function: {}", message)
            }
            MalError::SpecialForm(message) => {
                write!(f, "Error when evaluating special form: {}", message)
            }
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
    Nil,
    True,
    False,
    Number(f64),
    Symbol(String),
    Str(String),
}
