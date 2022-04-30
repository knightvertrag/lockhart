use std::io;
mod lexer;
mod repl;
mod token;
fn main() -> io::Result<()> {
    println!("Hello, Coompiler!");
    repl::start();

    Ok(())
}
