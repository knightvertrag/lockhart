use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::lexer::Lexer;
use crate::vm::Vm;

pub fn start() {
    let mut rl = Editor::<()>::new();
    let mut interpreter = Vm::init_vm();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
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
