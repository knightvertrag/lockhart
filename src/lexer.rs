use crate::token::TokenType;
use crate::token::{self, Num};
mod tests;
pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: u8,
}

impl Lexer {
    fn new(input: String) -> Lexer {
        let mut l = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: 0,
        };
        l.read_char();
        l
    }

    fn read_char(&mut self) {
        if self.read_position > self.input.len().try_into().unwrap() {
            self.ch = 0;
        } else {
            let inp = self.input.as_bytes();
            self.ch = inp[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn is_letter(ch: u8) -> bool {
        ch.is_ascii_alphabetic() || ch == '_' as u8
    }

    fn is_number(ch: u8) -> bool {
        ch.is_ascii_digit()
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }

    /// can read both identifiers and numbers based on f
    /// ## Arguments
    /// * `f` - checker function for digit or alphabet
    fn read_identifier(&mut self, f: fn(u8) -> bool) -> String {
        let position = self.position;
        while f(self.ch) {
            self.read_char();
        }
        self.input[position..self.position].to_string()
    }

    fn next_token(&mut self) -> token::Token {
        self.skip_whitespace();
        let res: token::Token;
        let current_char = (self.ch as char).to_string();
        if let Some(tok) = token::OPERATORS.get(&current_char) {
            res = token::Token::new(tok.clone(), current_char);
        } else if let Some(tok) = token::DELIMITERS.get(&current_char) {
            res = token::Token::new(tok.clone(), current_char);
        } else {
            if Lexer::is_letter(self.ch) {
                let literal = Lexer::read_identifier(self, Lexer::is_letter);
                let tok = token::Token::check_keyword(&literal);
                res = token::Token::new(tok, literal);
                return res;
            } else if Lexer::is_number(self.ch) {
                let literal = Lexer::read_identifier(self, Lexer::is_number);
                res = token::Token::new(TokenType::NUM(Num::NUM), literal);
                return res;
            } else {
                res = token::Token::new(TokenType::ILLEGAL, "".to_string());
            }
        }

        self.read_char();
        res
    }
}

