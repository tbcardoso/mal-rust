use crate::env::Env;
use crate::printer::pr_str;
use crate::reader::read_str;
use crate::types::MalValueType::{False, List, Nil, Number, RustFunc, Str, True, Vector};
use crate::types::{MalError, MalResult, MalValue, RustFunction};
use std::error::Error;
use std::fs;

pub fn ns() -> Vec<(&'static str, MalValue)> {
    vec![
        ("+", rust_func(add)),
        ("-", rust_func(subtract)),
        ("*", rust_func(multiply)),
        ("/", rust_func(divide)),
        ("prn", rust_func(prn)),
        ("println", rust_func(mal_println)),
        ("pr-str", rust_func(mal_pr_str)),
        ("str", rust_func(mal_str)),
        ("list", rust_func(list)),
        ("list?", rust_func(is_list)),
        ("empty?", rust_func(empty)),
        ("count", rust_func(count)),
        ("=", rust_func(equals)),
        ("<", rust_func(lt)),
        ("<=", rust_func(lte)),
        (">", rust_func(gt)),
        (">=", rust_func(gte)),
        ("read-string", rust_func(read_string)),
        ("slurp", rust_func(slurp)),
    ]
}

fn rust_func(func: fn(&[MalValue], &mut Env) -> MalResult) -> MalValue {
    MalValue::new(RustFunc(RustFunction(func)))
}

fn arg_count_eq(args: &[MalValue], expected: usize) -> Result<(), MalError> {
    if args.len() != expected {
        return Err(MalError::RustFunction(format!(
            "Expected {} argument{}, got {}",
            expected,
            if expected == 1 { "" } else { "s" },
            args.len()
        )));
    }

    Ok(())
}

fn get_number_arg(arg: &MalValue) -> Result<f64, MalError> {
    if let Number(n) = *arg.mal_type {
        Ok(n)
    } else {
        Err(MalError::RustFunction(
            "Argument must be a number".to_string(),
        ))
    }
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

    let arg_1 = get_number_arg(&args[0])?;
    let arg_2 = get_number_arg(&args[1])?;

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

fn lt(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 2)?;

    let arg_1 = get_number_arg(&args[0])?;
    let arg_2 = get_number_arg(&args[1])?;

    Ok(MalValue::new_boolean(arg_1 < arg_2))
}

fn lte(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 2)?;

    let arg_1 = get_number_arg(&args[0])?;
    let arg_2 = get_number_arg(&args[1])?;

    Ok(MalValue::new_boolean(arg_1 <= arg_2))
}

fn gt(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 2)?;

    let arg_1 = get_number_arg(&args[0])?;
    let arg_2 = get_number_arg(&args[1])?;

    Ok(MalValue::new_boolean(arg_1 > arg_2))
}

fn gte(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 2)?;

    let arg_1 = get_number_arg(&args[0])?;
    let arg_2 = get_number_arg(&args[1])?;

    Ok(MalValue::new_boolean(arg_1 >= arg_2))
}

fn pr_strs(strs: &[MalValue], print_readably: bool) -> Vec<String> {
    strs.iter().map(|arg| pr_str(arg, print_readably)).collect()
}

fn prn(args: &[MalValue], _env: &mut Env) -> MalResult {
    println!("{}", pr_strs(args, true).join(" "));

    Ok(MalValue::new(Nil))
}

fn mal_println(args: &[MalValue], _env: &mut Env) -> MalResult {
    println!("{}", pr_strs(args, false).join(" "));

    Ok(MalValue::new(Nil))
}

fn mal_pr_str(args: &[MalValue], _env: &mut Env) -> MalResult {
    Ok(MalValue::new(Str(pr_strs(args, true).join(" "))))
}

fn mal_str(args: &[MalValue], _env: &mut Env) -> MalResult {
    Ok(MalValue::new(Str(pr_strs(args, false).join(""))))
}

fn read_string(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if let Str(ref arg) = *args[0].mal_type {
        read_str(arg)
    } else {
        Err(MalError::RustFunction(
            "read_string expects argument to be of type String".to_string(),
        ))
    }
}

fn slurp(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if let Str(ref arg) = *args[0].mal_type {
        let file_content = fs::read_to_string(arg)
            .map_err(|e| MalError::RustFunction(format!("slurp: {}", e.description())))?;

        Ok(MalValue::new(Str(file_content)))
    } else {
        Err(MalError::RustFunction(
            "slurp expects argument to be of type String".to_string(),
        ))
    }
}
