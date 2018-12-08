use std::collections::HashMap;
use types::{MalError, MalResult, MalValue};

#[derive(Debug)]
struct Env {
    data: HashMap<String, MalValue>,
}

impl Env {
    fn new() -> Env {
        Env {
            data: HashMap::new(),
        }
    }

    fn set(&mut self, symbol_key: &str, val: MalValue) {
        self.data.insert(symbol_key.to_string(), val);
    }

    fn get(&self, symbol_key: &str) -> MalResult {
        self.data
            .get(symbol_key)
            .map(|val| val.clone())
            .ok_or_else(|| MalError::UndefinedSymbol(symbol_key.to_string()))
    }
}
