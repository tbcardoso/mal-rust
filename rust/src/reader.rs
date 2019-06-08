use crate::tokenizer::tokenize;
use crate::types::MalError::*;
use crate::types::MalTokenType;
use crate::types::MalValueType::*;
use crate::types::{MalError, MalMap, MalResult, MalToken, MalValue};

#[derive(Debug)]
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

    let mal_value = read_form(&mut reader)?;

    if reader.peek().is_some() {
        return Err(Parser("Expected EOF, found token".to_string()));
    }

    Ok(mal_value)
}

fn read_form(reader: &mut Reader) -> MalResult {
    match reader
        .peek()
        .ok_or_else(|| Parser("Unexpected EOF".to_string()))?
        .token_type
    {
        MalTokenType::LParen => read_list(reader),
        MalTokenType::LBracket => read_vector(reader),
        MalTokenType::LCurly => read_map(reader),
        MalTokenType::AtSign => read_short_form(reader, "deref"),
        MalTokenType::SingleQuote => read_short_form(reader, "quote"),
        MalTokenType::BackTick => read_short_form(reader, "quasiquote"),
        MalTokenType::Tilde => read_short_form(reader, "unquote"),
        MalTokenType::TildeAtSign => read_short_form(reader, "splice-unquote"),
        MalTokenType::Caret => read_with_meta(reader),
        _ => read_atom(reader),
    }
}

fn read_list(reader: &mut Reader) -> MalResult {
    Ok(MalValue::new_list(read_seq(reader, &MalTokenType::RParen)?))
}

fn read_vector(reader: &mut Reader) -> MalResult {
    Ok(MalValue::new_vector(read_seq(
        reader,
        &MalTokenType::RBracket,
    )?))
}

fn read_map(reader: &mut Reader) -> MalResult {
    Ok(MalValue::new(Map(MalMap::from_arguments(
        read_seq(reader, &MalTokenType::RCurly)?.as_slice(),
    )?)))
}

fn read_seq(reader: &mut Reader, end_token: &MalTokenType) -> Result<Vec<MalValue>, MalError> {
    reader.next().unwrap();

    let mut elems = Vec::new();

    loop {
        match reader
            .peek()
            .ok_or_else(|| Parser(format!("Expected '{:?}', got EOF", end_token).to_string()))?
            .token_type
        {
            ref t if t == end_token => {
                reader.next().unwrap();
                break;
            }
            _ => elems.push(read_form(reader)?),
        }
    }

    Ok(elems)
}

fn read_atom(reader: &mut Reader) -> MalResult {
    match reader
        .next()
        .ok_or_else(|| Parser("Unexpected EOF".to_string()))?
        .token_type
    {
        MalTokenType::Nil => Ok(MalValue::nil()),
        MalTokenType::True => Ok(MalValue::new(True)),
        MalTokenType::False => Ok(MalValue::new(False)),
        MalTokenType::Number(val) => Ok(MalValue::new(Number(val))),
        MalTokenType::Symbol(ref val) => Ok(MalValue::new(Symbol(val.clone()))),
        MalTokenType::Str(ref val) => Ok(MalValue::new(Str(val.clone()))),
        MalTokenType::Keyword(ref val) => Ok(MalValue::new(Keyword(val.clone()))),
        _ => Err(Parser("Unexpected token".to_string())),
    }
}

fn read_short_form(reader: &mut Reader, name: &str) -> MalResult {
    reader.next().unwrap();

    Ok(MalValue::new_list(vec![
        MalValue::new(Symbol(name.to_string())),
        read_form(reader)?,
    ]))
}

fn read_with_meta(reader: &mut Reader) -> MalResult {
    reader.next().unwrap();

    let meta = read_form(reader)?;
    let arg = read_form(reader)?;

    Ok(MalValue::new_list(vec![
        MalValue::new(Symbol("with-meta".to_string())),
        arg,
        meta,
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MalError;
    use crate::types::MalMap;
    use crate::types::MalTokenType;
    use crate::types::MalTokenType::{LParen, RParen};

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
    fn test_read_str_nil() {
        assert_eq!(read_str("nil"), Ok(MalValue::nil()));
    }

    #[test]
    fn test_read_str_true() {
        assert_eq!(read_str("true"), Ok(MalValue::new(True)));
    }

    #[test]
    fn test_read_str_false() {
        assert_eq!(read_str("false"), Ok(MalValue::new(False)));
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
        assert_eq!(read_str(r#""""#), Ok(MalValue::new(Str("".to_string()))));

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
    fn test_read_str_keyword() {
        assert_eq!(
            read_str(":abc"),
            Ok(MalValue::new(Keyword("abc".to_string())))
        );
        assert_eq!(read_str(":+"), Ok(MalValue::new(Keyword("+".to_string()))));
        assert_eq!(
            read_str(":abc_123_ABC"),
            Ok(MalValue::new(Keyword("abc_123_ABC".to_string())))
        );
    }

    #[test]
    fn test_read_str_list() {
        assert_eq!(read_str("()"), Ok(MalValue::new_list(Vec::new())));

        assert_eq!(
            read_str("(h)"),
            Ok(MalValue::new_list(vec![MalValue::new(Symbol(
                "h".to_string()
            )),]))
        );

        assert_eq!(
            read_str("(- xy 123.1)"),
            Ok(MalValue::new_list(vec![
                MalValue::new(Symbol("-".to_string())),
                MalValue::new(Symbol("xy".to_string())),
                MalValue::new(Number(123.1)),
            ]))
        );

        assert_eq!(
            read_str("(* (f (g) 1) 123)"),
            Ok(MalValue::new_list(vec![
                MalValue::new(Symbol("*".to_string())),
                MalValue::new_list(vec![
                    MalValue::new(Symbol("f".to_string())),
                    MalValue::new_list(vec![MalValue::new(Symbol("g".to_string())),]),
                    MalValue::new(Number(1.)),
                ]),
                MalValue::new(Number(123.)),
            ]))
        );

        match read_str("(h 12") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }
    }

    #[test]
    fn test_read_str_vector() {
        assert_eq!(read_str("[]"), Ok(MalValue::new_vector(Vec::new())));

        assert_eq!(
            read_str("[\"abc\"]"),
            Ok(MalValue::new_vector(vec![MalValue::new(Str(
                "abc".to_string()
            )),]))
        );

        assert_eq!(
            read_str("[x y 123.1]"),
            Ok(MalValue::new_vector(vec![
                MalValue::new(Symbol("x".to_string())),
                MalValue::new(Symbol("y".to_string())),
                MalValue::new(Number(123.1)),
            ]))
        );

        assert_eq!(
            read_str("[z [i [j] 5] 123]"),
            Ok(MalValue::new_vector(vec![
                MalValue::new(Symbol("z".to_string())),
                MalValue::new_vector(vec![
                    MalValue::new(Symbol("i".to_string())),
                    MalValue::new_vector(vec![MalValue::new(Symbol("j".to_string())),]),
                    MalValue::new(Number(5.)),
                ]),
                MalValue::new(Number(123.)),
            ]))
        );

        match read_str("[1 2") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }
    }

    #[test]
    fn test_read_str_hash_map() {
        assert_eq!(read_str("{}"), Ok(MalValue::new(Map(MalMap::new()))));

        assert_eq!(
            read_str("{\"a\" \"qwerty\"}"),
            Ok(MalValue::new(Map(MalMap::from_arguments(
                vec![
                    MalValue::new(Str("a".to_string())),
                    MalValue::new(Str("qwerty".to_string())),
                ]
                .as_slice()
            )
            .unwrap())))
        );

        assert_eq!(
            read_str("{:s1 {:s2 123}}"),
            Ok(MalValue::new(Map(MalMap::from_arguments(
                vec![
                    MalValue::new(Keyword("s1".to_string())),
                    MalValue::new(Map(MalMap::from_arguments(
                        vec![
                            MalValue::new(Keyword("s2".to_string())),
                            MalValue::new(Number(123.)),
                        ]
                        .as_slice()
                    )
                    .unwrap())),
                ]
                .as_slice()
            )
            .unwrap())))
        );

        match read_str("{:a 1 :b}") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }

        match read_str("{:a 1") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }
    }

    #[test]
    fn test_read_str_extra_tokens() {
        match read_str("aa 123") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }

        match read_str("(+ 1 x) (- 123 y)") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }
    }

    #[test]
    fn test_read_str_deref() {
        assert_eq!(
            read_str("@a"),
            Ok(MalValue::new_list(vec![
                MalValue::new(Symbol("deref".to_string())),
                MalValue::new(Symbol("a".to_string())),
            ]))
        );

        match read_str("@") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }
    }

    #[test]
    fn test_read_str_quote() {
        assert_eq!(
            read_str("'a"),
            Ok(MalValue::new_list(vec![
                MalValue::new(Symbol("quote".to_string())),
                MalValue::new(Symbol("a".to_string())),
            ]))
        );

        match read_str("'") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }
    }

    #[test]
    fn test_read_str_quasiquote() {
        assert_eq!(
            read_str("`a"),
            Ok(MalValue::new_list(vec![
                MalValue::new(Symbol("quasiquote".to_string())),
                MalValue::new(Symbol("a".to_string())),
            ]))
        );

        match read_str("`") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }
    }

    #[test]
    fn test_read_str_unquote() {
        assert_eq!(
            read_str("~a"),
            Ok(MalValue::new_list(vec![
                MalValue::new(Symbol("unquote".to_string())),
                MalValue::new(Symbol("a".to_string())),
            ]))
        );

        match read_str("~") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }
    }

    #[test]
    fn test_read_str_splice_unquote() {
        assert_eq!(
            read_str("~@a"),
            Ok(MalValue::new_list(vec![
                MalValue::new(Symbol("splice-unquote".to_string())),
                MalValue::new(Symbol("a".to_string())),
            ]))
        );

        match read_str("~@") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }
    }

    #[test]
    fn test_read_str_with_meta() {
        assert_eq!(
            read_str("^a +"),
            Ok(MalValue::new_list(vec![
                MalValue::new(Symbol("with-meta".to_string())),
                MalValue::new(Symbol("+".to_string())),
                MalValue::new(Symbol("a".to_string())),
            ]))
        );

        match read_str("^") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }

        match read_str("^a") {
            Err(MalError::Parser(_)) => {}
            _ => unreachable!("Expected Parser error."),
        }
    }
}
