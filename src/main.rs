use std::{env, io};
mod bytecode;
mod chunk;
mod compiler;
mod lexer;
mod repl;
mod token;
mod value;
mod vm;
use bytecode::Opcode;
use chunk::{Chunk, Lineno};
use vm::Vm;
fn main() -> io::Result<()> {
    // let args: Vec<String> = env::args().collect();
    // if args.len() == 1 {
    //     println!("Hello, Coompiler!");
    //     repl::start();
    // } else {
    //     let src_filename = &args[1];
    // }
    let s = "1 + 2*3";
    let mut interpreter = Vm::init_vm();
    interpreter.interpret(s.to_string());
    // let const_ = chunk.add_constant(value::Value::NUMBER(5.0));
    // chunk.write_chunk(Opcode::OPCONSTANT(const_), Lineno(0));
    // chunk.write_chunk(Opcode::OPNEGATE, Lineno(1));
    // chunk.write_chunk(Opcode::OPRETURN, Lineno(2));
    // interpreter.interpret(chunk);
    Ok(())
}
