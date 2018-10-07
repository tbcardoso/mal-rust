use types::MalValue;
use types::MalValueType::Number;

pub fn pr_str(mal_value: &MalValue) -> String {
    match mal_value.mal_type {
        Number(val) => val.to_string(),
        _ => "".to_string(),
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
}
