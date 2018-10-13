use types::MalValue;
use types::MalValueType::*;

pub fn pr_str(mal_value: &MalValue) -> String {
    match mal_value.mal_type {
        Number(val) => val.to_string(),
        Symbol(ref val) => val.clone(),
    }
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
}
