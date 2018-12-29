use crate::types::MalValue;
use crate::types::MalValueType::*;

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
        List(ref list) => pr_seq(list, "(", ")", print_readably),
        Vector(ref vec) => pr_seq(vec, "[", "]", print_readably),
        RustFunc(_) => "#<rust_function>".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RustFunction;

    #[test]
    fn test_pr_str_nil() {
        assert_eq!(pr_str(&MalValue::new(Nil), true), "nil");
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
    fn test_pr_str_list() {
        assert_eq!(pr_str(&MalValue::new(List(Vec::new())), true), "()");
        assert_eq!(
            pr_str(
                &MalValue::new(List(
                    vec![
                        MalValue::new(Symbol("+".to_string())),
                        MalValue::new(Number(456.)),
                        MalValue::new(Symbol("y".to_string())),
                    ]
                    .into_iter()
                    .collect()
                )),
                true
            ),
            "(+ 456 y)"
        );
    }

    #[test]
    fn test_pr_str_vector() {
        assert_eq!(pr_str(&MalValue::new(List(Vec::new())), true), "()");
        assert_eq!(
            pr_str(
                &MalValue::new(Vector(
                    vec![
                        MalValue::new(Symbol("x".to_string())),
                        MalValue::new(Number(456.)),
                        MalValue::new(Symbol("y".to_string())),
                    ]
                    .into_iter()
                    .collect()
                )),
                true
            ),
            "[x 456 y]"
        );
    }

    #[test]
    fn test_pr_str_rustfunc() {
        assert_eq!(
            pr_str(
                &MalValue::new(RustFunc(RustFunction(|_| Ok(MalValue::new(Number(0.)))))),
                true
            ),
            "#<rust_function>"
        );
    }
}
