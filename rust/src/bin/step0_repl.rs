extern crate malrs;

use malrs::readline::Readline;

fn main() {
    let mut readline = Readline::new();

    loop {
        match readline.readline() {
            None => break,
            Some(line) => {
                if !line.is_empty() {
                    println!("{}", rep(line.to_string()));
                }
            }
        }
    }
}

fn rep(s: String) -> String {
    print(eval(read(s)))
}

fn read(s: String) -> String {
    s
}

fn eval(s: String) -> String {
    s
}

fn print(s: String) -> String {
    s
}
