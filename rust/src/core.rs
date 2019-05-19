use crate::env::Env;
use crate::printer::pr_str;
use crate::reader::read_str;
use crate::types::MalValueType::{
    Atom, False, Keyword, List, MalFunc, Nil, Number, RustFunc, Str, Symbol, True, Vector,
};
use crate::types::{MalError, MalResult, MalValue};
use std::error::Error;
use std::fs;
use std::slice;

pub fn ns(env: &Env) -> Vec<(&'static str, MalValue)> {
    vec![
        ("+", MalValue::new_rust_func(add, env)),
        ("-", MalValue::new_rust_func(subtract, env)),
        ("*", MalValue::new_rust_func(multiply, env)),
        ("/", MalValue::new_rust_func(divide, env)),
        ("prn", MalValue::new_rust_func(prn, env)),
        ("println", MalValue::new_rust_func(mal_println, env)),
        ("pr-str", MalValue::new_rust_func(mal_pr_str, env)),
        ("str", MalValue::new_rust_func(mal_str, env)),
        ("list", MalValue::new_rust_func(list, env)),
        ("list?", MalValue::new_rust_func(is_list, env)),
        ("cons", MalValue::new_rust_func(cons, env)),
        ("concat", MalValue::new_rust_func(concat, env)),
        ("empty?", MalValue::new_rust_func(empty, env)),
        ("count", MalValue::new_rust_func(count, env)),
        ("nth", MalValue::new_rust_func(nth, env)),
        ("first", MalValue::new_rust_func(first, env)),
        ("rest", MalValue::new_rust_func(rest, env)),
        ("=", MalValue::new_rust_func(equals, env)),
        ("<", MalValue::new_rust_func(lt, env)),
        ("<=", MalValue::new_rust_func(lte, env)),
        (">", MalValue::new_rust_func(gt, env)),
        (">=", MalValue::new_rust_func(gte, env)),
        ("read-string", MalValue::new_rust_func(read_string, env)),
        ("slurp", MalValue::new_rust_func(slurp, env)),
        ("eval", MalValue::new_rust_func(mal_eval, env)),
        ("atom", MalValue::new_rust_func(atom, env)),
        ("atom?", MalValue::new_rust_func(is_atom, env)),
        ("deref", MalValue::new_rust_func(deref_atom, env)),
        ("reset!", MalValue::new_rust_func(reset_atom, env)),
        ("swap!", MalValue::new_rust_func(swap_atom, env)),
        ("throw", MalValue::new_rust_func(throw, env)),
        ("nil?", MalValue::new_rust_func(is_nil, env)),
        ("true?", MalValue::new_rust_func(is_true, env)),
        ("false?", MalValue::new_rust_func(is_false, env)),
        ("symbol?", MalValue::new_rust_func(is_symbol, env)),
        ("symbol", MalValue::new_rust_func(symbol, env)),
        ("keyword?", MalValue::new_rust_func(is_keyword, env)),
        ("keyword", MalValue::new_rust_func(keyword, env)),
        ("apply", MalValue::new_rust_func(apply, env)),
        ("map", MalValue::new_rust_func(map, env)),
    ]
}

static mut EVAL_FUNC: fn(ast: &MalValue, env: &mut Env) -> MalResult = dummy_eval;

fn dummy_eval(_: &MalValue, _: &mut Env) -> MalResult {
    panic!("core EVAL_FUNC was not set. You must call core::set_eval_func().")
}

pub fn set_eval_func(func: fn(ast: &MalValue, env: &mut Env) -> MalResult) {
    unsafe {
        EVAL_FUNC = func;
    }
}

fn core_eval(ast: &MalValue, env: &mut Env) -> MalResult {
    unsafe { EVAL_FUNC(ast, env) }
}

fn core_apply(function: &MalValue, args: &[MalValue], _env: &mut Env) -> MalResult {
    match *function.mal_type {
        RustFunc(ref rust_function) => {
            Ok((rust_function.func)(&args, &mut rust_function.env.clone())?)
        }
        MalFunc(ref mal_func) => {
            let mut func_env =
                Env::with_binds(Some(&mal_func.outer_env), &mal_func.parameters, &args)?;
            core_eval(&mal_func.body, &mut func_env)
        }
        _ => Err(MalError::RustFunction("Expected function.".to_string())),
    }
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

fn arg_count_gte(args: &[MalValue], min_args: usize) -> Result<(), MalError> {
    if args.len() < min_args {
        return Err(MalError::RustFunction(format!(
            "Expected at least {} argument{}, got {}",
            min_args,
            if min_args == 1 { "" } else { "s" },
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

fn cons(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 2)?;

    match *args[1].mal_type {
        List(ref vec) | Vector(ref vec) => {
            let mut new_vec = Vec::with_capacity(vec.len() + 1);
            new_vec.push(args[0].clone());
            new_vec.extend_from_slice(vec);

            Ok(MalValue::new(List(new_vec)))
        }
        _ => Err(MalError::RustFunction("Invalid 2nd argument".to_string())),
    }
}

fn concat(args: &[MalValue], _env: &mut Env) -> MalResult {
    let mut reult_vec = Vec::new();

    for arg in args {
        match *arg.mal_type {
            List(ref vec) | Vector(ref vec) => {
                reult_vec.extend_from_slice(vec);
            }
            _ => Err(MalError::RustFunction("Invalid argument".to_string()))?,
        }
    }

    Ok(MalValue::new(List(reult_vec)))
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

fn nth(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 2)?;

    let index = get_number_arg(&args[1])?;

    if let List(ref vec) | Vector(ref vec) = *args[0].mal_type {
        vec.get(index as usize)
            .cloned()
            .ok_or_else(|| MalError::RustFunction("nth: index out of range".to_string()))
    } else {
        Err(MalError::RustFunction("Invalid argument".to_string()))
    }
}

fn first(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    match *args[0].mal_type {
        List(ref vec) | Vector(ref vec) => Ok(vec.get(0).cloned().unwrap_or_else(MalValue::nil)),
        Nil => Ok(MalValue::nil()),
        _ => Err(MalError::RustFunction("Invalid argument".to_string())),
    }
}

fn rest(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    match *args[0].mal_type {
        List(ref vec) | Vector(ref vec) => Ok(if vec.is_empty() {
            MalValue::new(List(Vec::new()))
        } else {
            MalValue::new(List(Vec::from(&vec[1..])))
        }),
        Nil => Ok(MalValue::new(List(Vec::new()))),
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

    Ok(MalValue::nil())
}

fn mal_println(args: &[MalValue], _env: &mut Env) -> MalResult {
    println!("{}", pr_strs(args, false).join(" "));

    Ok(MalValue::nil())
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

fn mal_eval(args: &[MalValue], env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    core_eval(&args[0], env)
}

fn atom(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    Ok(MalValue::new_atom(args[0].clone()))
}

fn is_atom(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    Ok(MalValue::new_boolean(args[0].is_atom()))
}

fn deref_atom(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if let Atom(ref val) = *args[0].mal_type {
        Ok(val.borrow().clone())
    } else {
        Err(MalError::RustFunction(
            "Invalid argument. Expected atom.".to_string(),
        ))
    }
}

fn reset_atom(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 2)?;

    if let Atom(ref val) = *args[0].mal_type {
        val.replace(args[1].clone());
        Ok(args[1].clone())
    } else {
        Err(MalError::RustFunction(
            "Invalid argument. Expected atom.".to_string(),
        ))
    }
}

fn swap_atom(args: &[MalValue], env: &mut Env) -> MalResult {
    arg_count_gte(args, 2)?;

    let atom = if let Atom(ref val) = *args[0].mal_type {
        val
    } else {
        return Err(MalError::RustFunction(
            "Invalid 1st argument. Expected atom.".to_string(),
        ));
    };

    if !args[1].is_function() {
        return Err(MalError::RustFunction(
            "Invalid 2nd argument. Expected function.".to_string(),
        ));
    }

    let mut apply_args = Vec::with_capacity(args.len() - 1);
    apply_args.push(atom.borrow().clone());
    apply_args.extend_from_slice(&args[2..]);

    let result = core_apply(&args[1], &apply_args, env)?;

    atom.replace(result.clone());
    Ok(result)
}

fn throw(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_gte(args, 1)?;

    Err(MalError::Exception(args[0].clone()))
}

fn is_nil(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if let Nil = *args[0].mal_type {
        Ok(MalValue::new_boolean(true))
    } else {
        Ok(MalValue::new_boolean(false))
    }
}

fn is_true(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if let True = *args[0].mal_type {
        Ok(MalValue::new_boolean(true))
    } else {
        Ok(MalValue::new_boolean(false))
    }
}

fn is_false(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if let False = *args[0].mal_type {
        Ok(MalValue::new_boolean(true))
    } else {
        Ok(MalValue::new_boolean(false))
    }
}

fn is_symbol(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if let Symbol(_) = *args[0].mal_type {
        Ok(MalValue::new_boolean(true))
    } else {
        Ok(MalValue::new_boolean(false))
    }
}

fn apply(args: &[MalValue], env: &mut Env) -> MalResult {
    arg_count_gte(args, 2)?;

    let last_args_list = args.last().unwrap();

    if let List(ref last_args) | Vector(ref last_args) = *last_args_list.mal_type {
        let mut vec = Vec::with_capacity(args.len() + last_args.len() - 2);
        vec.extend_from_slice(&args[1..args.len() - 1]);
        vec.extend_from_slice(&last_args);

        core_apply(&args[0], &vec, env)
    } else {
        Err(MalError::RustFunction(
            "Invalid argument. Last argument of apply must be a list or vector.".to_string(),
        ))
    }
}

fn map(args: &[MalValue], env: &mut Env) -> MalResult {
    arg_count_eq(args, 2)?;

    let function = &args[0];

    if let List(ref vec) | Vector(ref vec) = *args[1].mal_type {
        let result_vec: Result<_, _> = vec
            .iter()
            .map(|elem| core_apply(function, slice::from_ref(elem), env))
            .collect();

        Ok(MalValue::new(List(result_vec?)))
    } else {
        Err(MalError::RustFunction(
            "Invalid argument. Second argument of map must be a list or vector.".to_string(),
        ))
    }
}

fn symbol(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if let Str(ref str_val) = *args[0].mal_type {
        Ok(MalValue::new(Symbol(str_val.clone())))
    } else {
        Err(MalError::RustFunction(
            "Argument must be a string.".to_string(),
        ))
    }
}

fn is_keyword(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if let Keyword(_) = *args[0].mal_type {
        Ok(MalValue::new_boolean(true))
    } else {
        Ok(MalValue::new_boolean(false))
    }
}

fn keyword(args: &[MalValue], _env: &mut Env) -> MalResult {
    arg_count_eq(args, 1)?;

    if let Str(ref str_val) = *args[0].mal_type {
        Ok(MalValue::new(Keyword(str_val.clone())))
    } else {
        Err(MalError::RustFunction(
            "Argument must be a string.".to_string(),
        ))
    }
}
