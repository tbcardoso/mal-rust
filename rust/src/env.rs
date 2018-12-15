use crate::types::{MalError, MalResult, MalValue};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Env {
    data: HashMap<String, MalValue>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, symbol_key: &str, val: MalValue) {
        self.data.insert(symbol_key.to_string(), val);
    }

    pub fn get(&self, symbol_key: &str) -> MalResult {
        self.data
            .get(symbol_key)
            .map(|val| val.clone())
            .ok_or_else(|| MalError::UndefinedSymbol(symbol_key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MalValueType::Str;

    #[test]
    fn test_get_from_empty_env() {
        let env = Env::new();

        assert_eq!(
            env.get("symbol"),
            Err(MalError::UndefinedSymbol("symbol".to_string()))
        );
    }

    #[test]
    fn test_set_and_get() {
        let mut env = Env::new();
        let val = MalValue::new(Str("abc".to_string()));

        env.set("sym", val.clone());

        assert_eq!(env.get("sym"), Ok(val));
    }
}
