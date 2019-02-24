use crate::env::Env;
use crate::printer::pr_str;
use crate::types::MalValueType::{False, List, Nil, Number, RustFunc, Str, True, Vector};
use crate::types::{MalError, MalResult, MalValue, RustFunction};

pub fn ns() -> Vec<(&'static str, MalValue)> {
    vec![
        ("+", rust_func(add)),
        ("-", rust_func(subtract)),
        ("*", rust_func(multiply)),
        ("/", rust_func(divide)),
        ("prn", rust_func(prn)),
        ("list", rust_func(list)),
        ("list?", rust_func(is_list)),
        ("empty?", rust_func(empty)),
        ("count", rust_func(count)),
        ("=", rust_func(equals)),
    ]
}

fn rust_func(func: fn(&[MalValue], &mut Env) -> MalResult) -> MalValue {
    MalValue::new(RustFunc(RustFunction(func)))
}

fn arg_count_eq(args: &[MalValue], expected: usize) -> Result<(), MalError> {
    if args.len() != expected {
        return Err(MalError::RustFunction(format!(
            "Expected {} arguments, got {}",
            expected,
            args.len()
        )));
    }

    Ok(())
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
    arg_count_eq(args, 2)?;

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

fn list(args: &[MalValue], _env: &mut Env) -> MalResult {
    Ok(MalValue::new(List(args.to_vec())))
}

fn is_list(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if args[0].is_list() {
        Ok(MalValue::new(True))
    } else {
        Ok(MalValue::new(False))
    }
}

fn empty(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    match *args[0].mal_type {
        List(ref vec) | Vector(ref vec) => {
            if vec.is_empty() {
                Ok(MalValue::new(True))
            } else {
                Ok(MalValue::new(False))
            }
        }
        Str(ref s) => {
            if s.is_empty() {
                Ok(MalValue::new(True))
            } else {
                Ok(MalValue::new(False))
            }
        }
        _ => Err(MalError::RustFunction("Invalid argument".to_string())),
    }
}

fn count(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    match *args[0].mal_type {
        List(ref vec) | Vector(ref vec) => Ok(MalValue::new(Number(vec.len() as f64))),
        Str(ref s) => Ok(MalValue::new(Number(s.len() as f64))),
        Nil => Ok(MalValue::new(Number(0.))),
        _ => Err(MalError::RustFunction("Invalid argument".to_string())),
    }
}

fn equals(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 2)?;

    Ok(MalValue::new_boolean(args[0] == args[1]))
}

fn prn(args: &[MalValue], _env: &mut Env) -> MalResult {
    let strs: Vec<_> = args.iter().map(|arg| pr_str(arg, true)).collect();

    println!("{}", strs.join(" "));

    Ok(MalValue::new(Nil))
}
