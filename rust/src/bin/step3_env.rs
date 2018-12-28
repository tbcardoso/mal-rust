use malrs::env::Env;
use malrs::printer::pr_str;
use malrs::reader::read_str;
use malrs::readline::Readline;
use malrs::types::MalValueType::{List, Number, RustFunc, Symbol};
use malrs::types::{MalError, MalResult, MalValue, RustFunction};

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
        MalValue::new(RustFunc(RustFunction(|args| {
            eval_arithmetic_operation(args, |a, b| a + b)
        }))),
    );

    env.set(
        "-",
        MalValue::new(RustFunc(RustFunction(|args| {
            eval_arithmetic_operation(args, |a, b| a - b)
        }))),
    );

    env.set(
        "*",
        MalValue::new(RustFunc(RustFunction(|args| {
            eval_arithmetic_operation(args, |a, b| a * b)
        }))),
    );

    env.set(
        "/",
        MalValue::new(RustFunc(RustFunction(|args| {
            eval_arithmetic_operation(args, |a, b| a / b)
        }))),
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

    let arg1 = if let Number(n) = *args.get(0).unwrap().mal_type {
        Ok(n)
    } else {
        Err(MalError::RustFunction(
            "First argument must be a number".to_string(),
        ))
    }?;

    let arg2 = if let Number(n) = *args.get(1).unwrap().mal_type {
        Ok(n)
    } else {
        Err(MalError::RustFunction(
            "Second argument must be a number".to_string(),
        ))
    }?;

    Ok(MalValue::new(Number(op(arg1, arg2))))
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
                            rust_function.0(&evaluated_list[1..])
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
    pr_str(mal_val)
}

fn eval_ast(ast: &MalValue, env: &Env) -> MalResult {
    match *ast.mal_type {
        Symbol(ref s) => env.get(&s),
        List(ref list) => {
            let evaluated_list: Result<_, _> =
                list.iter().map(|mal_val| eval(mal_val, env)).collect();

            Ok(MalValue::new(List(evaluated_list?)))
        }
        _ => Ok(ast.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use malrs::types::MalError::*;

    #[test]
    fn test_empty_program() {
        let mut env = create_env();

        assert_eq!(rep("", &mut env), Err(EmptyProgram));
    }

    #[test]
    fn test_empty_list() {
        let mut env = create_env();

        assert_eq!(rep("()", &mut env), Ok("()".to_string()));
    }

    #[test]
    fn test_nested_arithmetic() {
        let mut env = create_env();

        assert_eq!(rep("(+ 2 (* 3 4))", &mut env), Ok("14".to_string()));
    }
}
