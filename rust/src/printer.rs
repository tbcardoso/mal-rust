use std::collections::VecDeque;
use types::MalValue;
use types::MalValueType::*;

pub fn pr_str(mal_value: &MalValue) -> String {
    match *mal_value.mal_type {
        Number(val) => val.to_string(),
        Symbol(ref val) => val.clone(),
        Str(ref val) => escape_string(&val),
        List(ref list) => pr_list(list),
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

fn pr_list(list: &VecDeque<MalValue>) -> String {
    let elements: Vec<String> = list.iter().map(|val| pr_str(val)).collect();

    format!("({})", elements.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_str_number() {
        assert_eq!(pr_str(&MalValue::new(Number(123.))), "123");
        assert_eq!(pr_str(&MalValue::new(Number(-12.))), "-12");
        assert_eq!(pr_str(&MalValue::new(Number(7.5))), "7.5");
        assert_eq!(pr_str(&MalValue::new(Number(0.))), "0");
        assert_eq!(pr_str(&MalValue::new(Number(-12.3))), "-12.3");
    }

    #[test]
    fn test_pr_str_symbol() {
        assert_eq!(pr_str(&MalValue::new(Symbol("abc".to_string()))), "abc");
        assert_eq!(pr_str(&MalValue::new(Symbol("+".to_string()))), "+");
        assert_eq!(pr_str(&MalValue::new(Symbol("ab123".to_string()))), "ab123");
        assert_eq!(pr_str(&MalValue::new(Symbol("ab_CD".to_string()))), "ab_CD");
    }

    #[test]
    fn test_pr_str_str() {
        assert_eq!(pr_str(&MalValue::new(Str("".to_string()))), r#""""#);
        assert_eq!(pr_str(&MalValue::new(Str("abc".to_string()))), r#""abc""#);
        assert_eq!(
            pr_str(&MalValue::new(Str("ab 12 ABC".to_string()))),
            r#""ab 12 ABC""#
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("say 'something'".to_string()))),
            r#""say 'something'""#
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("123\nabc".to_string()))),
            r#""123\nabc""#
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("123\"abc".to_string()))),
            r#""123\"abc""#
        );
        assert_eq!(
            pr_str(&MalValue::new(Str("123\\abc".to_string()))),
            r#""123\\abc""#
        );
    }

    #[test]
    fn test_pr_str_list() {
        assert_eq!(pr_str(&MalValue::new(List(VecDeque::new()))), "()");
        assert_eq!(
            pr_str(&MalValue::new(List(
                vec![
                    MalValue::new(Symbol("+".to_string())),
                    MalValue::new(Number(456.)),
                    MalValue::new(Symbol("y".to_string())),
                ]
                .into_iter()
                .collect()
            ))),
            "(+ 456 y)"
        );
    }
}
