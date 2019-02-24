use crate::env::Env;
use crate::types::MalValueType::{Number, RustFunc};
use crate::types::{MalError, MalResult, MalValue, RustFunction};

pub fn ns() -> Vec<(&'static str, MalValue)> {
    vec![
        ("+", rust_func(add)),
        ("-", rust_func(subtract)),
        ("*", rust_func(multiply)),
        ("/", rust_func(divide)),
    ]
}

fn rust_func(func: fn(&[MalValue], &mut Env) -> MalResult) -> MalValue {
    MalValue::new(RustFunc(RustFunction(func)))
}

fn add(args: &[MalValue], _env: &mut Env) -> MalResult {
    eval_arithmetic_operation(args, |a, b| a + b)
}

fn subtract(args: &[MalValue], _env: &mut Env) -> MalResult {
    eval_arithmetic_operation(args, |a, b| a - b)
}

fn multiply(args: &[MalValue], _env: &mut Env) -> MalResult {
    eval_arithmetic_operation(args, |a, b| a * b)
}

fn divide(args: &[MalValue], _env: &mut Env) -> MalResult {
    eval_arithmetic_operation(args, |a, b| a / b)
}

fn eval_arithmetic_operation(args: &[MalValue], op: fn(f64, f64) -> f64) -> MalResult {
    if args.len() != 2 {
        return Err(MalError::RustFunction(format!(
            "Expected 2 arguments, got {}",
            args.len()
        )));
    }

    let arg_1 = if let Number(n) = *args[0].mal_type {
        Ok(n)
    } else {
        Err(MalError::RustFunction(
            "First argument must be a number".to_string(),
        ))
    }?;

    let arg_2 = if let Number(n) = *args[1].mal_type {
        Ok(n)
    } else {
        Err(MalError::RustFunction(
            "Second argument must be a number".to_string(),
        ))
    }?;

    Ok(MalValue::new(Number(op(arg_1, arg_2))))
}
