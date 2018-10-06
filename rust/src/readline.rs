use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};

pub struct Readline {
    editor: Editor<()>,
}

const HISTORY_FILE: &str = ".mal-history";
const PROMPT: &str = "user> ";

impl Readline {
    pub fn new() -> Readline {
        let config = Config::builder().auto_add_history(true).build();

        let mut editor = Editor::<()>::with_config(config);
        let _ = editor.load_history(HISTORY_FILE);

        Readline { editor }
    }

    pub fn readline(&mut self) -> Option<String> {
        let read_result = self.editor.readline(PROMPT);
        match read_result {
            Ok(line) => Some(line.trim().to_string()),
            Err(ReadlineError::Eof) => None,
            Err(err) => {
                println!("Error: {:?}", err);
                None
            }
        }
    }

    pub fn save_history(&self) {
        self.editor
            .save_history(HISTORY_FILE)
            .expect("Could not save command history.");
    }
}
