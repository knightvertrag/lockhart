use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::lexer::Lexer;

pub fn start() {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => lex(line.trim().to_string()),
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                panic!("{}", err);
            },
        }
    }
}

fn lex(line: String) {
    let lex = Lexer::new(line);
    for tok in lex {
        println!("{:?}", tok)
    }
}
