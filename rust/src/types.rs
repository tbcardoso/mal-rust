use std::fmt;
use types::MalError::*;

#[derive(Debug, PartialEq)]
pub struct MalValue {
    pub mal_type: MalValueType,
    // Possible extra fields: line, column
}

impl MalValue {
    pub fn new(mal_type: MalValueType) -> MalValue {
        MalValue { mal_type }
    }
}

#[derive(Debug, PartialEq)]
pub enum MalValueType {
    Number(f64),
    Symbol(String),
}

#[derive(Debug, PartialEq)]
pub enum MalError {
    EmptyProgram,
    Parser(String),
}

impl fmt::Display for MalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmptyProgram => write!(f, "Empty program."),
            Parser(message) => write!(f, "Parser error: {}", message),
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
    //StringLiteral(String),
}
