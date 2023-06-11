use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::lexer::Lexer;
use crate::vm::Vm;

pub fn start() {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let mut interpreter = Vm::init_vm();
                interpreter.interpret(line);
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                panic!("{}", err);
            }
        }
    }
}

fn lex(line: String) {
    let lex = Lexer::new(line);
    for tok in lex {
        println!("{:?}", tok)
    }
}
