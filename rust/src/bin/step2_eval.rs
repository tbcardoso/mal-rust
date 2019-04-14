use malrs::env::Env;
use malrs::printer::pr_str;
use malrs::reader::read_str;
use malrs::readline::Readline;
use malrs::types::MalValueType::{List, Map, Number, RustFunc, Symbol, Vector};
use malrs::types::{MalError, MalMap, MalResult, MalValue};
use std::iter::once;

fn main() {
    let env = create_env();
    let mut readline = Readline::new();

    loop {
        match readline.readline() {
            None => break,
            Some(line) => {
                if !line.is_empty() {
                    match rep(&line, &env) {
                        Ok(result) => println!("{}", result),
                        Err(MalError::EmptyProgram) => {}
                        Err(mal_error) => println!("Error! {}", mal_error),
                    }
                }
            }
        }
    }

    readline.save_history();
}

fn create_env() -> Env {
    let mut env = Env::new();

    env.set(
        "+",
        MalValue::new_rust_func(|args, _env| eval_arithmetic_operation(args, |a, b| a + b)),
    );

    env.set(
        "-",
        MalValue::new_rust_func(|args, _env| eval_arithmetic_operation(args, |a, b| a - b)),
    );

    env.set(
        "*",
        MalValue::new_rust_func(|args, _env| eval_arithmetic_operation(args, |a, b| a * b)),
    );

    env.set(
        "/",
        MalValue::new_rust_func(|args, _env| eval_arithmetic_operation(args, |a, b| a / b)),
    );

    env
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

fn rep(s: &str, env: &Env) -> Result<String, MalError> {
    let read_val = read(s)?;
    let eval_val = eval(&read_val, &env)?;
    Ok(print(&eval_val))
}

fn read(s: &str) -> MalResult {
    read_str(s)
}

fn eval(ast: &MalValue, env: &Env) -> MalResult {
    match *ast.mal_type {
        List(ref list) => {
            if list.is_empty() {
                Ok(ast.clone())
            } else {
                let evaluated_list_ast = eval_ast(ast, env)?;
                match *evaluated_list_ast.mal_type {
                    List(ref evaluated_list) => {
                        if let RustFunc(ref rust_function) = *evaluated_list
                            .get(0)
                            .expect("Evaluation of non-empty list resulted in empty list.")
                            .mal_type
                        {
                            rust_function.0(&evaluated_list[1..], &mut Env::new())
                        } else {
                            Err(MalError::Evaluation(
                                "First element of a list must evaluate to a function.".to_string(),
                            ))
                        }
                    }
                    _ => panic!(
                        "Evaluation of list resulted in non-list: {:?}",
                        evaluated_list_ast
                    ),
                }
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn print(mal_val: &MalValue) -> String {
    pr_str(mal_val, true)
}

fn eval_ast(ast: &MalValue, env: &Env) -> MalResult {
    match *ast.mal_type {
        Symbol(ref s) => env.get(&s),
        List(ref list) => Ok(MalValue::new(List(eval_ast_seq(list, env)?))),
        Vector(ref vec) => Ok(MalValue::new(Vector(eval_ast_seq(vec, env)?))),
        Map(ref mal_map) => eval_map(mal_map, env),
        _ => Ok(ast.clone()),
    }
}

fn eval_map(mal_map: &MalMap, env: &Env) -> MalResult {
    let map_args: Result<Vec<_>, _> = mal_map
        .iter()
        .flat_map(|(key, val)| once(Ok(key.clone())).chain(once(eval(val, env))))
        .collect();

    Ok(MalValue::new(Map(MalMap::from_arguments(
        map_args?.as_slice(),
    )?)))
}

fn eval_ast_seq(seq: &[MalValue], env: &Env) -> Result<Vec<MalValue>, MalError> {
    seq.iter().map(|mal_val| eval(mal_val, env)).collect()
}
