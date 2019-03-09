use crate::ApplyOkResult::{Return, TailCall};
use malrs::core::ns;
use malrs::env::Env;
use malrs::printer::pr_str;
use malrs::reader::read_str;
use malrs::readline::Readline;
use malrs::types::MalFunction;
use malrs::types::MalValueType;
use malrs::types::MalValueType::MalFunc;
use malrs::types::MalValueType::Nil;
use malrs::types::MalValueType::{List, Map, RustFunc, Symbol, Vector};
use malrs::types::{MalError, MalMap, MalResult, MalValue};
use std::iter::once;

fn main() {
    let mut env = create_root_env();
    let mut readline = Readline::new();

    rep("(def! not (fn* (a) (if a false true)))", &mut env).unwrap();

    loop {
        match readline.readline() {
            None => break,
            Some(line) => {
                if !line.is_empty() {
                    match rep(&line, &mut env) {
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

fn create_root_env() -> Env {
    let mut env = Env::new();

    for (name, val) in ns() {
        env.set(name, val);
    }

    env
}

fn rep(s: &str, env: &mut Env) -> Result<String, MalError> {
    let read_val = read(s)?;
    let eval_val = eval(&read_val, env)?;
    Ok(print(&eval_val))
}

fn read(s: &str) -> MalResult {
    read_str(s)
}

fn print(mal_val: &MalValue) -> String {
    pr_str(mal_val, true)
}

enum ApplyOkResult {
    Return(MalValue),
    TailCall(MalValue, Env),
}

type ApplyResult = Result<ApplyOkResult, MalError>;

fn eval(ast: &MalValue, env: &mut Env) -> MalResult {
    let mut cur_ast = ast.clone();
    let mut cur_env = env.clone();

    loop {
        match *cur_ast.mal_type {
            List(ref list) if list.is_empty() => return Ok(cur_ast.clone()),
            List(ref list) => {
                let first_arg = &list[0];

                let apply_result = match *first_arg.mal_type {
                    Symbol(ref name) if name == "def!" => {
                        apply_special_form_def(&list[1..], &mut cur_env)
                    }
                    Symbol(ref name) if name == "let*" => {
                        apply_special_form_let(&list[1..], &cur_env)
                    }
                    Symbol(ref name) if name == "fn*" => {
                        apply_special_form_fn(&list[1..], &cur_env)
                    }
                    Symbol(ref name) if name == "do" => {
                        apply_special_form_do(&list[1..], &mut cur_env)
                    }
                    Symbol(ref name) if name == "if" => {
                        apply_special_form_if(&list[1..], &mut cur_env)
                    }
                    _ => apply_ast(&cur_ast, &mut cur_env),
                }?;

                match apply_result {
                    Return(mal_value) => return Ok(mal_value),
                    TailCall(mal_val, new_env) => {
                        cur_ast = mal_val;
                        cur_env = new_env;
                    }
                }
            }
            _ => return eval_ast(&cur_ast, &mut cur_env),
        };
    }
}

fn eval_ast(ast: &MalValue, env: &mut Env) -> MalResult {
    match *ast.mal_type {
        Symbol(ref s) => env.get(&s),
        List(ref list) => Ok(MalValue::new(List(eval_ast_seq(list, env)?))),
        Vector(ref vec) => Ok(MalValue::new(Vector(eval_ast_seq(vec, env)?))),
        Map(ref mal_map) => eval_map(mal_map, env),
        _ => Ok(ast.clone()),
    }
}

fn eval_ast_seq(seq: &[MalValue], env: &mut Env) -> Result<Vec<MalValue>, MalError> {
    seq.iter().map(|mal_val| eval(mal_val, env)).collect()
}

fn eval_map(mal_map: &MalMap, env: &mut Env) -> MalResult {
    let map_args: Result<Vec<_>, _> = mal_map
        .iter()
        .flat_map(|(key, val)| once(Ok(key.clone())).chain(once(eval(val, env))))
        .collect();

    Ok(MalValue::new(Map(MalMap::from_arguments(
        map_args?.as_slice(),
    )?)))
}

fn apply_ast(ast: &MalValue, env: &mut Env) -> ApplyResult {
    let evaluated_list_ast = eval_ast(ast, env)?;
    match *evaluated_list_ast.mal_type {
        List(ref evaluated_list) => match *evaluated_list
            .get(0)
            .expect("Evaluation of non-empty list resulted in empty list.")
            .mal_type
        {
            RustFunc(ref rust_function) => Ok(Return(rust_function.0(&evaluated_list[1..], env)?)),
            MalFunc(ref mal_func) => {
                let func_env = Env::with_binds(
                    Some(&mal_func.outer_env),
                    &mal_func.parameters,
                    &evaluated_list[1..],
                )?;
                Ok(TailCall(mal_func.body.clone(), func_env))
            }
            _ => Err(MalError::Evaluation(
                "First element of a list must evaluate to a function.".to_string(),
            )),
        },
        _ => panic!(
            "Evaluation of list resulted in non-list: {:?}",
            evaluated_list_ast
        ),
    }
}

fn apply_special_form_def(args: &[MalValue], env: &mut Env) -> ApplyResult {
    if args.len() != 2 {
        return Err(MalError::SpecialForm(format!(
            "def! expected 2 arguments, got {}",
            args.len()
        )));
    }

    let arg1 = if let Symbol(ref symbol) = *args[0].mal_type {
        Ok(symbol)
    } else {
        Err(MalError::SpecialForm(
            "def! first argument must be a valid symbol name".to_string(),
        ))
    }?;

    let arg2 = eval(&args[1], env)?;

    env.set(arg1.as_str(), arg2.clone());

    Ok(Return(arg2))
}

fn apply_special_form_let(args: &[MalValue], env: &Env) -> ApplyResult {
    if args.len() != 2 {
        return Err(MalError::SpecialForm(format!(
            "let* expected 2 arguments, got {}",
            args.len()
        )));
    }

    let bindings = match *args[0].mal_type {
        List(ref bindings) | Vector(ref bindings) => Ok(bindings.as_slice()),
        _ => Err(MalError::SpecialForm(
            "let* first argument must be a list or a vector".to_string(),
        )),
    }?;

    if bindings.len() % 2 != 0 {
        return Err(MalError::SpecialForm(
            "let* bindings list must have an even number of elements".to_string(),
        ));
    }

    let mut inner_env = Env::with_outer_env(env);

    for i in (0..bindings.len()).step_by(2) {
        let binding_name = if let Symbol(ref symbol) = *bindings[i].mal_type {
            Ok(symbol)
        } else {
            Err(MalError::SpecialForm(
                "let* odd numbered elements of binding list must be valid symbol names".to_string(),
            ))
        }?;

        let binding_expr = eval(&bindings[i + 1], &mut inner_env)?;

        inner_env.set(binding_name.as_str(), binding_expr);
    }

    Ok(TailCall(args[1].clone(), inner_env))
}

fn apply_special_form_fn(args: &[MalValue], env: &Env) -> ApplyResult {
    if args.len() != 2 {
        return Err(MalError::SpecialForm(format!(
            "fn* expected 2 arguments, got {}",
            args.len()
        )));
    }

    let bindings = match *args[0].mal_type {
        List(ref bindings) | Vector(ref bindings) => Ok(bindings.as_slice()),
        _ => Err(MalError::SpecialForm(
            "fn* first argument must be a list or a vector".to_string(),
        )),
    }?;

    let parameters: Result<Vec<String>, _> = bindings
        .iter()
        .map(|val| {
            if let Symbol(ref symbol) = *val.mal_type {
                Ok(symbol.clone())
            } else {
                Err(MalError::SpecialForm(
                    "fn*! first argument must be a sequence of valid symbol names".to_string(),
                ))
            }
        })
        .collect();

    Ok(Return(MalValue::new(MalFunc(MalFunction {
        body: args[1].clone(),
        parameters: parameters?,
        outer_env: env.clone(),
    }))))
}

fn apply_special_form_do(args: &[MalValue], env: &mut Env) -> ApplyResult {
    if args.is_empty() {
        return Ok(Return(MalValue::new(Nil)));
    }

    for expr in args[..args.len() - 1].iter() {
        eval(expr, env)?;
    }

    Ok(TailCall(args.last().unwrap().clone(), env.clone()))
}

fn apply_special_form_if(args: &[MalValue], env: &mut Env) -> ApplyResult {
    if args.len() < 2 || args.len() > 3 {
        return Err(MalError::SpecialForm(format!(
            "if expected 2 or 3 arguments, got {}",
            args.len()
        )));
    }

    let test_result = eval(&args[0], env)?;

    match *test_result.mal_type {
        MalValueType::False | Nil => {
            if args.len() == 3 {
                Ok(TailCall(args[2].clone(), env.clone()))
            } else {
                Ok(Return(MalValue::new(Nil)))
            }
        }
        _ => Ok(TailCall(args[1].clone(), env.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use malrs::types::MalError::*;

    #[test]
    fn test_empty_program() {
        let mut env = create_root_env();
        assert_eq!(rep("", &mut env), Err(EmptyProgram));
    }

    #[test]
    fn test_empty_list() {
        let mut env = create_root_env();
        assert_eq!(rep("()", &mut env), Ok("()".to_string()));
    }

    #[test]
    fn test_empty_vector() {
        let mut env = create_root_env();
        assert_eq!(rep("[]", &mut env), Ok("[]".to_string()));
    }

    #[test]
    fn test_empty_map() {
        let mut env = create_root_env();
        assert_eq!(rep("{}", &mut env), Ok("{}".to_string()));
    }

    #[test]
    fn test_nested_arithmetic() {
        let mut env = create_root_env();
        assert_eq!(rep("(+ 2 (* 3 4))", &mut env), Ok("14".to_string()));
    }

    #[test]
    fn test_vector_eval() {
        let mut env = create_root_env();
        assert_eq!(rep("[1 2 (+ 1 2)]", &mut env), Ok("[1 2 3]".to_string()));
    }

    #[test]
    fn test_map_eval() {
        let mut env = create_root_env();
        assert_eq!(
            rep("{:a {:b (* 3 2)}}", &mut env),
            Ok("{:a {:b 6}}".to_string())
        );
    }

    #[test]
    fn test_special_form_def() {
        let mut env = create_root_env();
        assert_eq!(
            rep("(def! str1 \"abc\")", &mut env),
            Ok("\"abc\"".to_string())
        );
        assert_eq!(rep("str1", &mut env), Ok("\"abc\"".to_string()));
    }

    #[test]
    fn test_special_form_def_evaluates_2nd_par() {
        let mut env = create_root_env();
        assert_eq!(rep("(def! x (- 5 3))", &mut env), Ok("2".to_string()));
        assert_eq!(rep("x", &mut env), Ok("2".to_string()));
    }

    #[test]
    fn test_special_form_def_symbol_to_symbol() {
        let mut env = create_root_env();
        assert_eq!(rep("(def! x 1)", &mut env), Ok("1".to_string()));
        assert_eq!(rep("(def! y x)", &mut env), Ok("1".to_string()));
        assert_eq!(rep("x", &mut env), Ok("1".to_string()));
        assert_eq!(rep("y", &mut env), Ok("1".to_string()));
    }

    #[test]
    fn test_special_form_let() {
        let mut env = create_root_env();
        assert_eq!(rep("(let* (c 2) (+ 3 c))", &mut env), Ok("5".to_string()));
    }

    #[test]
    fn test_special_form_let_multiple_bindings() {
        let mut env = create_root_env();
        assert_eq!(
            rep("(let* (a 2 b (+ a a) c (- b a)) (+ (* a b) c))", &mut env),
            Ok("10".to_string())
        );
    }

    #[test]
    fn test_special_form_let_empty_bindings() {
        let mut env = create_root_env();
        assert_eq!(rep("(let* () 123)", &mut env), Ok("123".to_string()));
    }

    #[test]
    fn test_special_form_let_vector_bindings() {
        let mut env = create_root_env();
        assert_eq!(
            rep("(let* [a 2 b (+ a 1)] [a b (+ a b)])", &mut env),
            Ok("[2 3 5]".to_string())
        );
    }

    #[test]
    fn test_special_form_fn() {
        let mut env = create_root_env();
        assert_eq!(
            rep("(fn* [a b] (+ a b))", &mut env),
            Ok("#<function>".to_string())
        );
    }

    #[test]
    fn test_special_form_fn_eval() {
        let mut env = create_root_env();
        assert_eq!(
            rep("((fn* [a b] (+ a b)) 2 3)", &mut env),
            Ok("5".to_string())
        );
    }

    #[test]
    fn test_special_form_do() {
        let mut env = create_root_env();
        assert_eq!(rep("(do 1 :s2 3 :s4)", &mut env), Ok(":s4".to_string()));
    }

    #[test]
    fn test_special_form_do_empty() {
        let mut env = create_root_env();
        assert_eq!(rep("(do)", &mut env), Ok("nil".to_string()));
    }

    #[test]
    fn test_special_form_if() {
        let mut env = create_root_env();
        assert_eq!(rep("(if true 1 2)", &mut env), Ok("1".to_string()));
        assert_eq!(rep("(if true 2)", &mut env), Ok("2".to_string()));
        assert_eq!(rep("(if false 1 2)", &mut env), Ok("2".to_string()));
        assert_eq!(rep("(if nil :a :b)", &mut env), Ok(":b".to_string()));
        assert_eq!(rep("(if false :a)", &mut env), Ok("nil".to_string()));
    }
}
