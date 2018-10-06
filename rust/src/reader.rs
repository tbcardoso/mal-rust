use types::MalToken;

struct Reader {
    tokens: Vec<MalToken>,
    cur_pos: usize,
}

impl Reader {
    fn new(tokens: Vec<MalToken>) -> Reader {
        Reader { tokens, cur_pos: 0 }
    }
}

fn read_str(program: &str) {}

mod tokenizer {
    use regex::Regex;
    use types::MalToken;
    use types::MalTokenType;
    use types::MalTokenType::*;

    fn tokenize(program: &str) -> Vec<MalToken> {
        const TOKEN_RE_STR: &str =
            r##"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"|;.*|[^\s\[\]{}('"`,;)]*)"##;
        lazy_static! {
            static ref TOKEN_RE: Regex = Regex::new(TOKEN_RE_STR).unwrap();
        }

        let mut tokens: Vec<MalToken> = vec![];

        for capture in TOKEN_RE.captures_iter(program) {
            if let Some(token_type) = scan_token(&capture[1]) {
                tokens.push(MalToken::new(token_type))
            }
        }

        tokens
    }

    fn scan_token(text: &str) -> Option<MalTokenType> {
        match text.chars().next()? {
            '(' => Some(LParen),
            ')' => Some(RParen),
            '{' => Some(LCurly),
            '}' => Some(RCurly),
            ']' => Some(RBracket),
            '[' => Some(LBracket),
            ';' => None,
            //'"' => Some(StringLiteral),
            _ => scan_nonspecial_token(text),
        }
    }

    fn scan_nonspecial_token(text: &str) -> Option<MalTokenType> {
        const NUMBER_RE_STR: &str = r#"^-?\d+\.?\d*$"#;
        lazy_static! {
            static ref NUMBER_RE: Regex = Regex::new(NUMBER_RE_STR).unwrap();
        }

        if NUMBER_RE.is_match(&text) {
            return Some(NumberToken(
                text.parse()
                    .expect(&format!("Error parsing number: {}", text)),
            ));
        }

        Some(SymbolToken(text.to_string()))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_tokenize_empty() {
            assert_eq!(tokenize(""), vec![]);
        }

        #[test]
        fn test_tokenize_blanks() {
            assert_eq!(tokenize("   "), vec![]);
            assert_eq!(tokenize("\t"), vec![]);
            assert_eq!(tokenize("\n"), vec![]);
            assert_eq!(tokenize("\t  \t\n "), vec![]);
        }

        #[test]
        fn test_tokenize_comma() {
            assert_eq!(tokenize(","), vec![]);
            assert_eq!(tokenize(",,,,"), vec![]);
        }

        #[test]
        fn test_tokenize_comments() {
            assert_eq!(tokenize(";"), vec![]);
            assert_eq!(tokenize(";;;;;;"), vec![]);
            assert_eq!(tokenize(";abc 123 qwe123 ()"), vec![]);
        }

        #[test]
        fn test_tokenize_parens() {
            assert_eq!(tokenize("("), vec![MalToken::new(LParen)]);
            assert_eq!(tokenize(")"), vec![MalToken::new(RParen)]);
            assert_eq!(
                tokenize("()"),
                vec![MalToken::new(LParen), MalToken::new(RParen)]
            );
            assert_eq!(
                tokenize("))((())"),
                vec![
                    MalToken::new(RParen),
                    MalToken::new(RParen),
                    MalToken::new(LParen),
                    MalToken::new(LParen),
                    MalToken::new(LParen),
                    MalToken::new(RParen),
                    MalToken::new(RParen),
                ]
            );
        }

        #[test]
        fn test_tokenize_curly_brackets() {
            assert_eq!(tokenize("{"), vec![MalToken::new(LCurly)]);
            assert_eq!(tokenize("}"), vec![MalToken::new(RCurly)]);
            assert_eq!(
                tokenize("{}"),
                vec![MalToken::new(LCurly), MalToken::new(RCurly)]
            );
            assert_eq!(
                tokenize("{{}}{{"),
                vec![
                    MalToken::new(LCurly),
                    MalToken::new(LCurly),
                    MalToken::new(RCurly),
                    MalToken::new(RCurly),
                    MalToken::new(LCurly),
                    MalToken::new(LCurly),
                ]
            );
        }

        #[test]
        fn test_tokenize_square_brackets() {
            assert_eq!(tokenize("["), vec![MalToken::new(LBracket)]);
            assert_eq!(tokenize("]"), vec![MalToken::new(RBracket)]);
            assert_eq!(
                tokenize("[]"),
                vec![MalToken::new(LBracket), MalToken::new(RBracket)]
            );
            assert_eq!(
                tokenize("][[]]]"),
                vec![
                    MalToken::new(RBracket),
                    MalToken::new(LBracket),
                    MalToken::new(LBracket),
                    MalToken::new(RBracket),
                    MalToken::new(RBracket),
                    MalToken::new(RBracket),
                ]
            );
        }

        #[test]
        fn test_tokenize_numbers() {
            assert_eq!(tokenize("1"), vec![MalToken::new(NumberToken(1.))]);
            assert_eq!(tokenize("-1"), vec![MalToken::new(NumberToken(-1.))]);
            assert_eq!(
                tokenize("123456"),
                vec![MalToken::new(NumberToken(123456.))]
            );
            assert_eq!(
                tokenize("-12 0 53 -5"),
                vec![
                    MalToken::new(NumberToken(-12.)),
                    MalToken::new(NumberToken(0.)),
                    MalToken::new(NumberToken(53.)),
                    MalToken::new(NumberToken(-5.)),
                ]
            );
        }

        #[test]
        fn test_tokenize_symbols() {
            assert_eq!(
                tokenize("a"),
                vec![MalToken::new(SymbolToken("a".to_string()))]
            );
            assert_eq!(
                tokenize("ab_c123"),
                vec![MalToken::new(SymbolToken("ab_c123".to_string()))]
            );
            assert_eq!(
                tokenize("*"),
                vec![MalToken::new(SymbolToken("*".to_string()))]
            );
            assert_eq!(
                tokenize("qwer - a0b +bc"),
                vec![
                    MalToken::new(SymbolToken("qwer".to_string())),
                    MalToken::new(SymbolToken("-".to_string())),
                    MalToken::new(SymbolToken("a0b".to_string())),
                    MalToken::new(SymbolToken("+bc".to_string())),
                ]
            );
        }
    }
}
