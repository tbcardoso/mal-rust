use crate::types::MalError;
use crate::types::MalToken;
use crate::types::MalTokenType;
use crate::types::MalTokenType::*;
use lazy_static::lazy_static;
use regex::Regex;

pub fn tokenize(program: &str) -> Result<Vec<MalToken>, MalError> {
    const TOKEN_RE_STR: &str =
        r##"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"##;
    lazy_static! {
        static ref TOKEN_RE: Regex = Regex::new(TOKEN_RE_STR).unwrap();
    }

    let mut tokens: Vec<MalToken> = vec![];

    for capture in TOKEN_RE.captures_iter(program) {
        if let Some(token_type) = scan_token(&capture[1])? {
            tokens.push(MalToken::new(token_type))
        }
    }

    Ok(tokens)
}

fn scan_token(text: &str) -> Result<Option<MalTokenType>, MalError> {
    match text
        .chars()
        .next()
        .ok_or_else(|| MalError::Tokenizer("Unexpected EOF".to_string()))?
    {
        '(' => Ok(Some(LParen)),
        ')' => Ok(Some(RParen)),
        '{' => Ok(Some(LCurly)),
        '}' => Ok(Some(RCurly)),
        ']' => Ok(Some(RBracket)),
        '[' => Ok(Some(LBracket)),
        '@' => Ok(Some(AtSign)),
        '\'' => Ok(Some(SingleQuote)),
        '`' => Ok(Some(BackTick)),
        '~' => Ok(Some(if text == "~@" { TildeAtSign } else { Tilde })),
        ';' => Ok(None),
        '"' => Ok(Some(Str(scan_string(text)?))),
        ':' => Ok(Some(scan_keyword(text))),
        _ => Ok(Some(scan_nonspecial_token(text)?)),
    }
}

fn scan_string(text: &str) -> Result<String, MalError> {
    let mut unescaped_str = String::new();

    let mut chars = text.chars();
    chars.next().unwrap();

    loop {
        match chars.next() {
            Some('\"') => break,
            Some('\\') => {
                unescaped_str.push(unescape_char(chars.next().ok_or_else(|| {
                    MalError::Tokenizer("Expected '\"', got EOF".to_string())
                })?))
            }
            Some(c) => unescaped_str.push(c),
            None => return Err(MalError::Tokenizer("Expected '\"', got EOF".to_string())),
        }
    }

    Ok(unescaped_str.to_string())
}

fn unescape_char(c: char) -> char {
    match c {
        'n' => '\n',
        other => other,
    }
}

fn scan_keyword(text: &str) -> MalTokenType {
    Keyword(text[1..].to_string())
}

fn scan_nonspecial_token(text: &str) -> Result<MalTokenType, MalError> {
    let reserved_name = match text {
        "nil" => Some(Nil),
        "true" => Some(True),
        "false" => Some(False),
        _ => None,
    };

    if reserved_name.is_some() {
        return Ok(reserved_name.unwrap());
    }

    const NUMBER_RE_STR: &str = r#"^-?\d+\.?\d*$"#;
    lazy_static! {
        static ref NUMBER_RE: Regex = Regex::new(NUMBER_RE_STR).unwrap();
    }

    if NUMBER_RE.is_match(&text) {
        return Ok(Number(
            text.parse()
                .unwrap_or_else(|_| panic!("Error parsing number: {}", text)),
        ));
    }

    Ok(Symbol(text.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_empty() {
        assert_eq!(tokenize(""), Ok(vec![]));
    }

    #[test]
    fn test_tokenize_blanks() {
        assert_eq!(tokenize("   "), Ok(vec![]));
        assert_eq!(tokenize("\t"), Ok(vec![]));
        assert_eq!(tokenize("\n"), Ok(vec![]));
        assert_eq!(tokenize("\t  \t\n "), Ok(vec![]));
    }

    #[test]
    fn test_tokenize_comma() {
        assert_eq!(tokenize(","), Ok(vec![]));
        assert_eq!(tokenize(",,,,"), Ok(vec![]));
    }

    #[test]
    fn test_tokenize_comments() {
        assert_eq!(tokenize(";"), Ok(vec![]));
        assert_eq!(tokenize(";;;;;;"), Ok(vec![]));
        assert_eq!(tokenize(";abc 123 qwe123 ()"), Ok(vec![]));
    }

    #[test]
    fn test_tokenize_parens() {
        assert_eq!(tokenize("("), Ok(vec![MalToken::new(LParen)]));
        assert_eq!(tokenize(")"), Ok(vec![MalToken::new(RParen)]));
        assert_eq!(
            tokenize("()"),
            Ok(vec![MalToken::new(LParen), MalToken::new(RParen)])
        );
        assert_eq!(
            tokenize("))((())"),
            Ok(vec![
                MalToken::new(RParen),
                MalToken::new(RParen),
                MalToken::new(LParen),
                MalToken::new(LParen),
                MalToken::new(LParen),
                MalToken::new(RParen),
                MalToken::new(RParen),
            ])
        );
    }

    #[test]
    fn test_tokenize_curly_brackets() {
        assert_eq!(tokenize("{"), Ok(vec![MalToken::new(LCurly)]));
        assert_eq!(tokenize("}"), Ok(vec![MalToken::new(RCurly)]));
        assert_eq!(
            tokenize("{}"),
            Ok(vec![MalToken::new(LCurly), MalToken::new(RCurly)])
        );
        assert_eq!(
            tokenize("{{}}{{"),
            Ok(vec![
                MalToken::new(LCurly),
                MalToken::new(LCurly),
                MalToken::new(RCurly),
                MalToken::new(RCurly),
                MalToken::new(LCurly),
                MalToken::new(LCurly),
            ])
        );
    }

    #[test]
    fn test_tokenize_square_brackets() {
        assert_eq!(tokenize("["), Ok(vec![MalToken::new(LBracket)]));
        assert_eq!(tokenize("]"), Ok(vec![MalToken::new(RBracket)]));
        assert_eq!(
            tokenize("[]"),
            Ok(vec![MalToken::new(LBracket), MalToken::new(RBracket)])
        );
        assert_eq!(
            tokenize("][[]]]"),
            Ok(vec![
                MalToken::new(RBracket),
                MalToken::new(LBracket),
                MalToken::new(LBracket),
                MalToken::new(RBracket),
                MalToken::new(RBracket),
                MalToken::new(RBracket),
            ])
        );
    }

    #[test]
    fn test_tokenize_at_sign() {
        assert_eq!(tokenize("@"), Ok(vec![MalToken::new(AtSign)]));
        assert_eq!(
            tokenize("@@ @"),
            Ok(vec![
                MalToken::new(AtSign),
                MalToken::new(AtSign),
                MalToken::new(AtSign)
            ])
        );
    }

    #[test]
    fn test_tokenize_single_quote() {
        assert_eq!(tokenize("'"), Ok(vec![MalToken::new(SingleQuote)]));
        assert_eq!(
            tokenize("' ''"),
            Ok(vec![
                MalToken::new(SingleQuote),
                MalToken::new(SingleQuote),
                MalToken::new(SingleQuote)
            ])
        );
    }

    #[test]
    fn test_tokenize_backtick() {
        assert_eq!(tokenize("`"), Ok(vec![MalToken::new(BackTick)]));
        assert_eq!(
            tokenize("`` `"),
            Ok(vec![
                MalToken::new(BackTick),
                MalToken::new(BackTick),
                MalToken::new(BackTick)
            ])
        );
    }

    #[test]
    fn test_tokenize_tilde() {
        assert_eq!(tokenize("~"), Ok(vec![MalToken::new(Tilde)]));
        assert_eq!(
            tokenize("~~ ~"),
            Ok(vec![
                MalToken::new(Tilde),
                MalToken::new(Tilde),
                MalToken::new(Tilde)
            ])
        );
    }

    #[test]
    fn test_tokenize_tilde_at_sign() {
        assert_eq!(tokenize("~@"), Ok(vec![MalToken::new(TildeAtSign)]));
        assert_eq!(
            tokenize("~@~@ ~@"),
            Ok(vec![
                MalToken::new(TildeAtSign),
                MalToken::new(TildeAtSign),
                MalToken::new(TildeAtSign)
            ])
        );
    }

    #[test]
    fn test_tokenize_nil() {
        assert_eq!(tokenize("nil"), Ok(vec![MalToken::new(Nil)]));
    }

    #[test]
    fn test_tokenize_true() {
        assert_eq!(tokenize("true"), Ok(vec![MalToken::new(True)]));
    }

    #[test]
    fn test_tokenize_false() {
        assert_eq!(tokenize("false"), Ok(vec![MalToken::new(False)]));
    }

    #[test]
    fn test_tokenize_numbers() {
        assert_eq!(tokenize("1"), Ok(vec![MalToken::new(Number(1.))]));
        assert_eq!(tokenize("-1"), Ok(vec![MalToken::new(Number(-1.))]));
        assert_eq!(
            tokenize("123456"),
            Ok(vec![MalToken::new(Number(123_456.))])
        );
        assert_eq!(tokenize("12.2"), Ok(vec![MalToken::new(Number(12.2))]));
        assert_eq!(
            tokenize("-123.99"),
            Ok(vec![MalToken::new(Number(-123.99))])
        );
        assert_eq!(tokenize("80."), Ok(vec![MalToken::new(Number(80.))]));
        assert_eq!(tokenize("-2."), Ok(vec![MalToken::new(Number(-2.))]));
        assert_eq!(
            tokenize("-12 0 53.2 -5."),
            Ok(vec![
                MalToken::new(Number(-12.)),
                MalToken::new(Number(0.)),
                MalToken::new(Number(53.2)),
                MalToken::new(Number(-5.)),
            ])
        );
    }

    #[test]
    fn test_tokenize_symbols() {
        assert_eq!(
            tokenize("a"),
            Ok(vec![MalToken::new(Symbol("a".to_string()))])
        );
        assert_eq!(
            tokenize("ab_c123"),
            Ok(vec![MalToken::new(Symbol("ab_c123".to_string()))])
        );
        assert_eq!(
            tokenize("*"),
            Ok(vec![MalToken::new(Symbol("*".to_string()))])
        );
        assert_eq!(
            tokenize("qwer - a0b +bc"),
            Ok(vec![
                MalToken::new(Symbol("qwer".to_string())),
                MalToken::new(Symbol("-".to_string())),
                MalToken::new(Symbol("a0b".to_string())),
                MalToken::new(Symbol("+bc".to_string())),
            ])
        );
    }

    #[test]
    fn test_tokenize_strings() {
        assert_eq!(
            tokenize(r#""""#),
            Ok(vec![MalToken::new(Str("".to_string()))])
        );
        assert_eq!(
            tokenize(r#""abc""#),
            Ok(vec![MalToken::new(Str("abc".to_string()))])
        );
        assert_eq!(
            tokenize(r#""abc 123  ab""#),
            Ok(vec![MalToken::new(Str("abc 123  ab".to_string()))])
        );
        assert_eq!(
            tokenize(r#""quotes 'aa'""#),
            Ok(vec![MalToken::new(Str("quotes 'aa'".to_string()))])
        );
        assert_eq!(
            tokenize(r#""123\nab""#),
            Ok(vec![MalToken::new(Str("123\nab".to_string()))])
        );
        assert_eq!(
            tokenize(r#""ab\"cd""#),
            Ok(vec![MalToken::new(Str("ab\"cd".to_string()))])
        );
        assert_eq!(
            tokenize(r#""ab\\cd""#),
            Ok(vec![MalToken::new(Str("ab\\cd".to_string()))])
        );

        match tokenize(r#""abc"#) {
            Err(MalError::Tokenizer(_)) => {}
            _ => unreachable!("Expected Tokenizer error."),
        }

        match tokenize(r#""abc\"#) {
            Err(MalError::Tokenizer(_)) => {}
            _ => unreachable!("Expected Tokenizer error."),
        }
    }

    #[test]
    fn test_tokenize_keywords() {
        assert_eq!(
            tokenize(":a"),
            Ok(vec![MalToken::new(Keyword("a".to_string()))])
        );

        assert_eq!(
            tokenize(":ab12"),
            Ok(vec![MalToken::new(Keyword("ab12".to_string()))])
        );
    }
}
