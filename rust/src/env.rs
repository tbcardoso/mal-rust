use crate::types::MalValueType::List;
use crate::types::{MalError, MalResult, MalValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct Env(Rc<EnvImpl>);

#[derive(Debug, PartialEq)]
struct EnvImpl {
    data: RefCell<HashMap<String, MalValue>>,
    outer: Option<Env>,
}

fn create_env(outer: Option<&Env>) -> Env {
    Env(Rc::new(EnvImpl {
        data: RefCell::new(HashMap::new()),
        outer: outer.cloned(),
    }))
}

impl Env {
    pub fn new() -> Env {
        create_env(None)
    }

    pub fn with_outer_env(outer: &Env) -> Env {
        create_env(Some(outer))
    }

    pub fn with_binds<S: AsRef<str>>(
        outer: Option<&Env>,
        binds: &[S],
        exprs: &[MalValue],
    ) -> Result<Env, MalError> {
        let mut env = create_env(outer);

        for (i, bind) in binds.iter().enumerate() {
            if bind.as_ref() == "&" {
                if binds.len() <= (i + 1) {
                    return Err(MalError::Evaluation(
                        "Error in argument binding: no parameter after '&'".to_string(),
                    ));
                }

                env.set(
                    binds[i + 1].as_ref(),
                    MalValue::new(List(exprs[i..].to_vec())),
                );

                break;
            }

            env.set(
                bind.as_ref(),
                exprs.get(i).cloned().unwrap_or_else(MalValue::nil),
            )
        }

        Ok(env)
    }

    pub fn set(&mut self, symbol_key: &str, val: MalValue) {
        self.0.data.borrow_mut().insert(symbol_key.to_string(), val);
    }

    pub fn find(&self, symbol_key: &str) -> Option<Env> {
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

impl Default for Env {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn test_with_binds_empty() {
        let mut env1 = Env::new();
        let val = MalValue::new(Str("abc".to_string()));
        env1.set("sym", val.clone());

        let env2 = Env::with_binds::<&str>(Some(&env1), &[], &[]).unwrap();

        assert_eq!(env2.get("sym"), Ok(val));
    }

    #[test]
    fn test_envs_same_outer() {
        let mut env1 = Env::new();
        let val1 = MalValue::new(Str("abc".to_string()));
        env1.set("sym", val1.clone());

        let val2 = MalValue::new(Number(1.));
        let env2 = Env::with_binds(Some(&env1), &["sym"], &[val2.clone()]).unwrap();

        let env3 = Env::with_outer_env(&env1);

        assert_eq!(env2.get("sym"), Ok(val2));
        assert_eq!(env3.get("sym"), Ok(val1));
    }

    #[test]
    fn test_with_binds() {
        let val1 = MalValue::new(Number(1.));
        let val2 = MalValue::new(Str("abc".to_string()));

        let env = Env::with_binds(
            None,
            &["s1".to_string(), "s2".to_string()],
            &[val1.clone(), val2.clone()],
        )
        .unwrap();

        assert_eq!(env.get("s1"), Ok(val1));
        assert_eq!(env.get("s2"), Ok(val2));
    }

    #[test]
    fn test_with_binds_extra_exprs() {
        let val1 = MalValue::new(Number(1.));
        let val2 = MalValue::new(Str("abc".to_string()));
        let val3 = MalValue::new(Str("xyz".to_string()));

        let env = Env::with_binds(
            None,
            &["s1", "s2"],
            &[val1.clone(), val2.clone(), val3.clone()],
        )
        .unwrap();

        assert_eq!(env.get("s1"), Ok(val1));
        assert_eq!(env.get("s2"), Ok(val2));
    }

    #[test]
    fn test_with_binds_extra_binds() {
        let val1 = MalValue::new(Number(1.));
        let val2 = MalValue::new(Str("abc".to_string()));

        let env = Env::with_binds(
            None,
            &["s1", "s2", "s3", "s4"],
            &[val1.clone(), val2.clone()],
        )
        .unwrap();

        assert_eq!(env.get("s1"), Ok(val1));
        assert_eq!(env.get("s2"), Ok(val2));
        assert_eq!(env.get("s3"), Ok(MalValue::nil()));
        assert_eq!(env.get("s4"), Ok(MalValue::nil()));
    }

    #[test]
    fn test_with_binds_variadic() {
        let val1 = MalValue::new(Number(1.));
        let val2 = MalValue::new(Str("abc".to_string()));
        let val3 = MalValue::new(Number(2.));

        let env = Env::with_binds(
            None,
            &["s1", "&", "v"],
            &[val1.clone(), val2.clone(), val3.clone()],
        )
        .unwrap();

        assert_eq!(env.get("s1"), Ok(val1));
        assert_eq!(env.get("v"), Ok(MalValue::new(List(vec![val2, val3,]))));
    }

    #[test]
    fn test_with_binds_variadic_only() {
        let val1 = MalValue::new(Number(1.));
        let val2 = MalValue::new(Str("abc".to_string()));
        let val3 = MalValue::new(Number(2.));

        let env = Env::with_binds(
            None,
            &["&", "v"],
            &[val1.clone(), val2.clone(), val3.clone()],
        )
        .unwrap();

        assert_eq!(
            env.get("v"),
            Ok(MalValue::new(List(vec![val1, val2, val3,])))
        );
    }
}
