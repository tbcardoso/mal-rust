use crate::types::{MalError, MalResult, MalValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Env(Rc<EnvImpl>);

#[derive(Debug)]
struct EnvImpl {
    data: RefCell<HashMap<String, MalValue>>,
    outer: Option<Env>,
}

fn create_env(outer: Option<Env>) -> Env {
    Env(Rc::new(EnvImpl {
        data: RefCell::new(HashMap::new()),
        outer,
    }))
}

impl Env {
    pub fn new() -> Env {
        create_env(None)
    }

    pub fn with_outer_env(outer: &Env) -> Env {
        create_env(Some(outer.clone()))
    }

    pub fn set(&mut self, symbol_key: &str, val: MalValue) {
        self.0.data.borrow_mut().insert(symbol_key.to_string(), val);
    }

    fn find(&self, symbol_key: &str) -> Option<Env> {
        if self.0.data.borrow().contains_key(symbol_key) {
            Some(self.clone())
        } else {
            match &self.0.outer {
                Some(outer) => outer.find(symbol_key),
                None => None,
            }
        }
    }

    pub fn get(&self, symbol_key: &str) -> MalResult {
        self.find(symbol_key)
            .map(|env| env.0.data.borrow().get(symbol_key).unwrap().clone())
            .ok_or_else(|| MalError::UndefinedSymbol(symbol_key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MalValueType::{Number, Str};

    fn undefined_symbol_err(symbol_key: &str) -> MalResult {
        Err(MalError::UndefinedSymbol(symbol_key.to_string()))
    }

    #[test]
    fn test_undefined_symbol() {
        let env1 = Env::new();

        assert_eq!(env1.get("symbol1"), undefined_symbol_err("symbol1"));

        let mut env2 = Env::with_outer_env(&env1);

        env2.set("symbol2", MalValue::new(Str("abc".to_string())));

        assert_eq!(env2.get("symbol1"), undefined_symbol_err("symbol1"));
    }

    #[test]
    fn test_set_and_get() {
        let mut env = Env::new();
        let val = MalValue::new(Str("abc".to_string()));

        env.set("sym", val.clone());

        assert_eq!(env.get("sym"), Ok(val));
    }

    #[test]
    fn test_get_symbol_from_outer_env() {
        let mut env1 = Env::new();
        let val = MalValue::new(Str("abc".to_string()));
        env1.set("sym", val.clone());

        let env2 = Env::with_outer_env(&env1);

        assert_eq!(env2.get("sym"), Ok(val));
    }

    #[test]
    fn test_symbol_hiding() {
        let val1 = MalValue::new(Number(1.));
        let val2 = MalValue::new(Number(2.));

        let mut env1 = Env::new();
        env1.set("sym1", val1.clone());

        let mut env2 = Env::with_outer_env(&env1);
        env2.set("sym1", val2.clone());

        assert_eq!(env1.get("sym1"), Ok(val1));
        assert_eq!(env2.get("sym1"), Ok(val2));
    }

}
