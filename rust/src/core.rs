use crate::types::MalValueType::{Number, RustFunc};
use crate::types::{MalError, MalResult, MalValue, RustFunction};

pub fn ns() -> Vec<(&'static str, MalValue)> {
    vec![
        (
            "+",
            MalValue::new(RustFunc(RustFunction(|args| {
                eval_arithmetic_operation(args, |a, b| a + b)
            }))),
        ),
        (
            "-",
            MalValue::new(RustFunc(RustFunction(|args| {
                eval_arithmetic_operation(args, |a, b| a - b)
            }))),
        ),
        (
            "*",
            MalValue::new(RustFunc(RustFunction(|args| {
                eval_arithmetic_operation(args, |a, b| a * b)
            }))),
        ),
        (
            "/",
            MalValue::new(RustFunc(RustFunction(|args| {
                eval_arithmetic_operation(args, |a, b| a / b)
            }))),
        ),
    ]
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
