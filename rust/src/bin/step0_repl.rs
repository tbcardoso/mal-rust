use std::io;
use std::io::Write;

fn main() {
    loop {
        print!("user> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .expect("Failed to read line");

        print!("{}", rep(input));
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
