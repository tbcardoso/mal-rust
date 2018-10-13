use std::collections::VecDeque;
use tokenizer::tokenize;
use types::MalError::*;
use types::MalTokenType;
use types::MalTokenType::LParen;
use types::MalValueType::*;
use types::{MalResult, MalToken, MalValue};

struct Reader {
    tokens: Vec<MalToken>,
    cur_pos: usize,
}

impl Reader {
    fn new(tokens: Vec<MalToken>) -> Reader {
        Reader { tokens, cur_pos: 0 }
    }

    fn next(&mut self) -> Option<&MalToken> {
        let token = self.tokens.get(self.cur_pos)?;
        self.cur_pos += 1;
        Some(token)
    }

    fn peek(&self) -> Option<&MalToken> {
        self.tokens.get(self.cur_pos)
    }
}

pub fn read_str(program: &str) -> MalResult {
    let tokens = tokenize(program)?;

    if tokens.is_empty() {
        return Err(EmptyProgram);
    }

    let mut reader = Reader::new(tokens);

    read_form(&mut reader)
}

fn read_form(reader: &mut Reader) -> MalResult {
    match reader
        .peek()
        .ok_or_else(|| Parser("Unexpected EOF".to_string()))?
        .token_type
    {
        LParen => read_list(reader),
        _ => read_atom(reader),
    }
}

fn read_list(reader: &mut Reader) -> MalResult {
    reader.next().unwrap();

    let mut elems = VecDeque::new();

    loop {
        match reader
            .peek()
            .ok_or_else(|| Parser("Expected ')', got EOF".to_string()))?
            .token_type
        {
            MalTokenType::RParen => {
                reader.next().unwrap();
                break;
            }
            _ => elems.push_back(read_form(reader)?),
        }
    }

    Ok(MalValue::new(List(elems)))
}

fn read_atom(reader: &mut Reader) -> MalResult {
    match reader
        .next()
        .ok_or_else(|| Parser("Unexpected EOF".to_string()))?
        .token_type
    {
        MalTokenType::Number(val) => Ok(MalValue::new(Number(val))),
        MalTokenType::Symbol(ref val) => Ok(MalValue::new(Symbol(val.clone()))),
        MalTokenType::Str(ref val) => Ok(MalValue::new(Str(val.clone()))),
        _ => Err(Parser("Unexpected token".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::MalError;
    use types::MalTokenType;
    use types::MalTokenType::{LParen, RParen};

    #[test]
    fn test_reader() {
        let mut reader = Reader::new(vec![
            MalToken::new(LParen),
            MalToken::new(MalTokenType::Symbol("+".to_string())),
            MalToken::new(MalTokenType::Number(2.)),
            MalToken::new(MalTokenType::Symbol("x".to_string())),
            MalToken::new(RParen),
        ]);

        assert_eq!(reader.peek(), Some(&MalToken::new(LParen)));
        assert_eq!(reader.next(), Some(&MalToken::new(LParen)));

        assert_eq!(
            reader.peek(),
            Some(&MalToken::new(MalTokenType::Symbol("+".to_string())))
        );
        assert_eq!(
            reader.next(),
            Some(&MalToken::new(MalTokenType::Symbol("+".to_string())))
        );

        assert_eq!(
            reader.peek(),
            Some(&MalToken::new(MalTokenType::Number(2.)))
        );
        assert_eq!(
            reader.next(),
            Some(&MalToken::new(MalTokenType::Number(2.)))
        );

        assert_eq!(
            reader.peek(),
            Some(&MalToken::new(MalTokenType::Symbol("x".to_string())))
        );
        assert_eq!(
            reader.next(),
            Some(&MalToken::new(MalTokenType::Symbol("x".to_string())))
        );

        assert_eq!(reader.peek(), Some(&MalToken::new(RParen)));
        assert_eq!(reader.next(), Some(&MalToken::new(RParen)));

        assert_eq!(reader.peek(), None);
        assert_eq!(reader.next(), None);

        assert_eq!(reader.peek(), None);
        assert_eq!(reader.next(), None);
    }

    #[test]
    fn test_read_str_empty_program() {
        assert_eq!(read_str(""), Err(EmptyProgram));
        assert_eq!(read_str("  \t \n  "), Err(EmptyProgram));
        assert_eq!(read_str("; this is a comment"), Err(EmptyProgram));
    }

    #[test]
    fn test_read_str_number() {
        assert_eq!(read_str("123"), Ok(MalValue::new(Number(123.))));
        assert_eq!(read_str("-12"), Ok(MalValue::new(Number(-12.))));
        assert_eq!(read_str("-5.5"), Ok(MalValue::new(Number(-5.5))));
        assert_eq!(read_str("10."), Ok(MalValue::new(Number(10.))));
    }

    #[test]
    fn test_read_str_symbol() {
        assert_eq!(
            read_str("abc"),
            Ok(MalValue::new(Symbol("abc".to_string())))
        );
        assert_eq!(read_str("+"), Ok(MalValue::new(Symbol("+".to_string()))));
        assert_eq!(
            read_str("abc_123_ABC"),
            Ok(MalValue::new(Symbol("abc_123_ABC".to_string())))
        );
    }

    #[test]
    fn test_read_str_string() {
        assert_eq!(
            read_str(r#""""#),
            Ok(MalValue::new(Str("".to_string())))
        );

        assert_eq!(
            read_str(r#""abc""#),
            Ok(MalValue::new(Str("abc".to_string())))
        );

        assert_eq!(
            read_str(r#""abc\n123""#),
            Ok(MalValue::new(Str("abc\n123".to_string())))
        );
    }

    #[test]
    fn test_read_str_list() {
        assert_eq!(read_str("()"), Ok(MalValue::new(List(VecDeque::new()))));

        assert_eq!(
            read_str("(h)"),
            Ok(MalValue::new(List(
                vec![MalValue::new(Symbol("h".to_string())),]
                    .into_iter()
                    .collect()
            )))
        );

        assert_eq!(
            read_str("(- xy 123.1)"),
            Ok(MalValue::new(List(
                vec![
                    MalValue::new(Symbol("-".to_string())),
                    MalValue::new(Symbol("xy".to_string())),
                    MalValue::new(Number(123.1)),
                ].into_iter()
                .collect()
            )))
        );

        assert_eq!(
            read_str("(* (f (g) 1) 123)"),
            Ok(MalValue::new(List(
                vec![
                    MalValue::new(Symbol("*".to_string())),
                    MalValue::new(List(
                        vec![
                            MalValue::new(Symbol("f".to_string())),
                            MalValue::new(List(
                                vec![MalValue::new(Symbol("g".to_string())),]
                                    .into_iter()
                                    .collect()
                            )),
                            MalValue::new(Number(1.)),
                        ].into_iter()
                        .collect()
                    )),
                    MalValue::new(Number(123.)),
                ].into_iter()
                .collect()
            )))
        );

        match read_str("(h 12") {
            Err(MalError::Parser(_)) => {}
            _ => assert!(false, "Expected Parser error."),
        }
    }
}
