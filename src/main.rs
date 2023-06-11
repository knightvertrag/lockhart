use std::{env, io};
mod bytecode;
mod chunk;
mod compiler;
mod lexer;
mod repl;
mod token;
mod value;
mod vm;
mod object;
use bytecode::Opcode;
use chunk::{Chunk, Lineno};
use vm::Vm;
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("===============Lockhart initiated===============");
        repl::start();
    } else {
        let src_filename = &args[1];
    }
    // let s = "4 == nil";
    // let mut interpreter = Vm::init_vm();
    // interpreter.interpret(s.to_string());
    Ok(())
}
