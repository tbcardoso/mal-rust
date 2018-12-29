use malrs::printer::pr_str;
use malrs::reader::read_str;
use malrs::readline::Readline;
use malrs::types::MalError;
use malrs::types::MalResult;
use malrs::types::MalValue;

fn main() {
    let mut readline = Readline::new();

    loop {
        match readline.readline() {
            None => break,
            Some(line) => {
                if !line.is_empty() {
                    match rep(&line) {
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

fn rep(s: &str) -> Result<String, MalError> {
    let read_val = read(s)?;
    let eval_val = eval(&read_val)?;
    Ok(print(&eval_val))
}

fn read(s: &str) -> MalResult {
    read_str(s)
}

fn eval(mal_val: &MalValue) -> Result<&MalValue, MalError> {
    Ok(mal_val)
}

fn print(mal_val: &MalValue) -> String {
    pr_str(mal_val, true)
}
