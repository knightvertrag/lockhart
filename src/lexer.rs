use std::cell::RefCell;

use crate::token::{self, Num, Token};
use crate::token::{Operators, TokenType};
mod tests;
pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: u8,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
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
        if self.read_position >= self.input.len() {
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

    fn peek_ahead(&self) -> Option<u8> {
        if self.read_position >= self.input.len() {
            return None;
        }
        Some(self.input.as_bytes()[self.read_position])
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

    pub fn next_token(&mut self) -> token::Token {
        let selfcell = RefCell::new(self);
        selfcell.borrow_mut().skip_whitespace();
        let mut res = token::Token::new(TokenType::ILLEGAL, "".to_string());

        let current_char = (selfcell.borrow().ch as char).to_string();
        if let Some(tok) = token::OPERATORS.get(&current_char) {
            let mut build_double = |t, next_ch: char, lit: &str| {
                if selfcell.borrow().peek_ahead() == Some(next_ch as u8) {
                    let mut literal = lit.to_string();
                    literal.push(next_ch);
                    res = token::Token::new(t, literal);
                    selfcell.borrow_mut().read_char();
                }
                else {
                    res = Token::new(tok.clone(), lit.to_string());
                }
            };
            match tok {
                TokenType::OPERATORS(Operators::ASSIGN) => {
                    build_double(TokenType::OPERATORS(Operators::EQ), '=', "=");
                }
                TokenType::OPERATORS(Operators::GT) => {
                    build_double(TokenType::OPERATORS(Operators::GEQ), '=', ">");
                }
                TokenType::OPERATORS(Operators::LT) => {
                    build_double(TokenType::OPERATORS(Operators::LEQ), '=', "<");
                }
                TokenType::OPERATORS(Operators::NOT) => {
                    build_double(TokenType::OPERATORS(Operators::NEQ), '=', "!");
                }
                _ => {
                    res = token::Token::new(tok.clone(), current_char);
                }
            }
        } else if let Some(tok) = token::DELIMITERS.get(&current_char) {
            res = token::Token::new(tok.clone(), current_char);
        } else {
            if Lexer::is_letter(selfcell.borrow().ch) {
                let literal = Lexer::read_identifier(*selfcell.borrow_mut(), Lexer::is_letter);
                let tok = token::Token::check_keyword(&literal);
                res = token::Token::new(tok, literal);
                return res;
            } else if Lexer::is_number(selfcell.borrow().ch) {
                let literal = Lexer::read_identifier(*selfcell.borrow_mut(), Lexer::is_number);
                res = token::Token::new(TokenType::NUM(Num::NUM), literal);
                return res;
            } else {
                res = token::Token::new(TokenType::ILLEGAL, "".to_string());
            }
        }

        selfcell.borrow_mut().read_char();
        res
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.read_position > self.input.len() {
            return None;
        }
        Some(self.next_token())
    }
}
