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
fn main() -> io::Result<()> {
    // let args: Vec<String> = env::args().collect();
    // if args.len() == 1 {
    //     println!("Hello, Coompiler!");
    //     repl::start();
    // } else {
    //     let src_filename = &args[1];
    // }
    
    Ok(())
}
