use std::io;
mod lexer;
mod repl;
mod token;
mod parser;
fn main() -> io::Result<()> {
    println!("Hello, Coompiler!");
    repl::start();

    Ok(())
}
