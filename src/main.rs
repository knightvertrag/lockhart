use std::{io, env};
mod lexer;
mod repl;
mod token;
mod bytecode;
mod chunk;
mod value;
mod vm;
use chunk::Chunk;
use bytecode::Opcode;
use vm::Vm;
fn main() -> io::Result<()> {
    // let args: Vec<String> = env::args().collect();
    // if args.len() == 1 {
    //     println!("Hello, Coompiler!");
    //     repl::start();
    // } else {
    //     let src_filename = &args[1];
    // }
    let mut interpreter = Vm::init_vm();
    let mut chunk = Chunk::new();
    chunk.code.push((Opcode::OPCONSTANT(0), chunk::Lineno(1)));
    chunk.add_constant_double(3.0);
    chunk.code.push((Opcode::OPCONSTANT(1), chunk::Lineno(2)));
    chunk.add_constant_double(4.0);
    chunk.code.push((Opcode::OPADD, chunk::Lineno(3)));
    chunk.code.push((Opcode::OPRETURN, chunk::Lineno(4)));
    interpreter.interpret(chunk);
    Ok(())
}
