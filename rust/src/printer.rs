use crate::types::MalMap;
use crate::types::MalValue;
use crate::types::MalValueType::*;
use std::iter::once;

pub fn pr_str(mal_value: &MalValue, print_readably: bool) -> String {
    match *mal_value.mal_type {
        Nil => "nil".to_string(),
        True => "true".to_string(),
        False => "false".to_string(),
        Number(val) => val.to_string(),
        Symbol(ref val) => val.clone(),
        Str(ref val) => {
            if print_readably {
                escape_string(&val)
            } else {
                val.to_string()
            }
        }
        Keyword(ref val) => format!(":{}", val),
        List(ref list) => pr_seq(list, "(", ")", print_readably),
        Vector(ref vec) => pr_seq(vec, "[", "]", print_readably),
        Map(ref mal_map) => pr_map(mal_map, print_readably),
        RustFunc(_) => "#<rust_function>".to_string(),
        MalFunc(_) => "#<function>".to_string(),
        Atom(ref val) => format!("(atom {})", pr_str(&(*val.borrow()), print_readably)),
    }
}

fn escape_string(text: &str) -> String {
    let mut escaped_str = String::new();
    let mut chars = text.chars();

    loop {
        match chars.next() {
            None => break,
            Some('\\') => escaped_str.push_str("\\\\"),
            Some('\n') => escaped_str.push_str("\\n"),
            Some('"') => escaped_str.push_str("\\\""),
            Some(c) => escaped_str.push(c),
        }
    }

    format!("\"{}\"", escaped_str)
}

fn pr_seq(list: &[MalValue], start: &str, end: &str, print_readably: bool) -> String {
    let elements: Vec<String> = list.iter().map(|val| pr_str(val, print_readably)).collect();

    format!("{}{}{}", start, elements.join(" "), end)
}

fn pr_map(mal_map: &MalMap, print_readably: bool) -> String {
    let map_args: Vec<_> = mal_map
        .iter()
        .flat_map(|(key, val)| once(key.clone()).chain(once(val.clone())))
        .collect();

    pr_seq(map_args.as_slice(), "{", "}", print_readably)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Env;
    use crate::types::MalMap;

    #[test]
    fn test_pr_str_nil() {
        assert_eq!(pr_str(&MalValue::nil(), true), "nil");
    }

    #[test]
    fn test_pr_str_true() {
        assert_eq!(pr_str(&MalValue::new(True), true), "true");
    }

    #[test]
    fn test_pr_str_false() {
        assert_eq!(pr_str(&MalValue::new(False), true), "false");
    }

    #[test]
    fn test_pr_str_number() {
        assert_eq!(pr_str(&MalValue::new(Number(123.)), true), "123");
        assert_eq!(pr_str(&MalValue::new(Number(-12.)), true), "-12");
        assert_eq!(pr_str(&MalValue::new(Number(7.5)), true), "7.5");
        assert_eq!(pr_str(&MalValue::new(Number(0.)), true), "0");
        assert_eq!(pr_str(&MalValue::new(Number(-12.3)), true), "-12.3");
    }

    #[test]
    fn test_pr_str_symbol() {
        assert_eq!(
            pr_str(&MalValue::new(Symbol("abc".to_string())), true),
            "abc"
        );
        assert_eq!(pr_str(&MalValue::new(Symbol("+".to_string())), true), "+");
        assert_eq!(
            pr_str(&MalValue::new(Symbol("ab123".to_string())), true),
            "ab123"
        );
        assert_eq!(
            pr_str(&MalValue::new(Symbol("ab_CD".to_string())), true),
            "ab_CD"
        );
    }

    #[test]
    fn test_pr_str_str_readably() {
        assert_eq!(pr_str(&MalValue::new(Str("".to_string())), true), r#""""#);
        assert_eq!(
            pr_str(&MalValue::new(Str("abc".to_string())), true),
            r#""abc""#
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("ab 12 ABC".to_string())), true),
            r#""ab 12 ABC""#
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("say 'something'".to_string())), true),
            r#""say 'something'""#
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("123\nabc".to_string())), true),
            r#""123\nabc""#
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("123\"abc".to_string())), true),
            r#""123\"abc""#
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("123\\abc".to_string())), true),
            r#""123\\abc""#
        );
    }

    #[test]
    fn test_pr_str_str_not_readably() {
        assert_eq!(pr_str(&MalValue::new(Str("".to_string())), false), "");
        assert_eq!(pr_str(&MalValue::new(Str("abc".to_string())), false), "abc");
        assert_eq!(
            pr_str(&MalValue::new(Str("ab 12 ABC".to_string())), false),
            "ab 12 ABC"
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("say 'something'".to_string())), false),
            "say 'something'"
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("123\nabc".to_string())), false),
            "123\nabc"
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("123\"abc".to_string())), false),
            "123\"abc"
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("123\\abc".to_string())), false),
            "123\\abc"
        );
    }

    #[test]
    fn test_pr_str_keyword() {
        assert_eq!(pr_str(&MalValue::new(Keyword("a".to_string())), true), ":a");
        assert_eq!(
            pr_str(&MalValue::new(Keyword("abc123".to_string())), true),
            ":abc123"
        );
    }

    #[test]
    fn test_pr_str_list() {
        assert_eq!(pr_str(&MalValue::new(List(Vec::new())), true), "()");
        assert_eq!(
            pr_str(
                &MalValue::new(List(vec![
                    MalValue::new(Symbol("+".to_string())),
                    MalValue::new(Number(456.)),
                    MalValue::new(Symbol("y".to_string())),
                ])),
                true,
            ),
            "(+ 456 y)"
        );
    }

    #[test]
    fn test_pr_str_vector() {
        assert_eq!(pr_str(&MalValue::new(List(Vec::new())), true), "()");
        assert_eq!(
            pr_str(
                &MalValue::new(Vector(vec![
                    MalValue::new(Symbol("x".to_string())),
                    MalValue::new(Number(456.)),
                    MalValue::new(Symbol("y".to_string())),
                ])),
                true,
            ),
            "[x 456 y]"
        );
    }

    #[test]
    fn test_pr_str_hashmap() {
        assert_eq!(pr_str(&MalValue::new(Map(MalMap::new())), true), "{}");

        assert_eq!(
            pr_str(
                &MalValue::new(Map(MalMap::from_arguments(&[
                    MalValue::new(Keyword("a".to_string())),
                    MalValue::new(Map(MalMap::from_arguments(&[
                        MalValue::new(Str("b".to_string())),
                        MalValue::new(Number(12.)),
                    ])
                    .unwrap())),
                ])
                .unwrap())),
                true,
            ),
            "{:a {\"b\" 12}}"
        );
    }

    #[test]
    fn test_pr_str_rustfunc() {
        assert_eq!(
            pr_str(
                &MalValue::new_rust_func(|_, _| Ok(MalValue::new(Number(0.))), &Env::new()),
                true,
            ),
            "#<rust_function>"
        );
    }

    #[test]
    fn test_pr_str_malfunc() {
        assert_eq!(
            pr_str(
                &MalValue::new_mal_func(MalValue::nil(), Vec::new(), Env::new()),
                true
            ),
            "#<function>"
        );
    }

    #[test]
    fn test_pr_str_atom() {
        assert_eq!(
            pr_str(&MalValue::new_atom(MalValue::new(Number(123.))), true),
            "(atom 123)"
        )
    }
}
