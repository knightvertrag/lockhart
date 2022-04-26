use crate::token;

pub struct Lexer {
    input: String,
    position: u32,
    read_position: u32,
    ch: char,
}

impl Lexer {
    fn new(input: String) -> Lexer {
        Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: '.',
        }
    }
}
#[cfg(test)]
fn testTokenNext() {
    let x = token::Token {
        _type: token::NUM.to_string(),
        literal: "Yolo".to_string(),
    };
}
