use std::collections::VecDeque;
use types::MalValue;
use types::MalValueType::*;

pub fn pr_str(mal_value: &MalValue) -> String {
    match mal_value.mal_type {
        Number(val) => val.to_string(),
        Symbol(ref val) => val.clone(),
        List(ref list) => pr_list(list),
    }
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
    fn test_pr_str_list() {
        assert_eq!(pr_str(&MalValue::new(List(VecDeque::new()))), "()");
        assert_eq!(
            pr_str(&MalValue::new(List(
                vec![
                    MalValue::new(Symbol("+".to_string())),
                    MalValue::new(Number(456.)),
                    MalValue::new(Symbol("y".to_string())),
                ].into_iter()
                .collect()
            ))),
            "(+ 456 y)"
        );
    }
}
