use crate::token::TokenType;
use crate::token::{self, Token};
mod tests;
#[derive(Debug)]
pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: u8,
    lineno: usize,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let mut l = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: 0,
            lineno: 1,
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
            if self.ch as char == '\n' {
                self.lineno += 1;
            }
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

    fn token(&self, type_: TokenType, literal: String, lineno: usize, start: usize, end: usize) -> Token {
        Token::new(type_, literal, lineno, start, end)
    }

    fn read_identifier(&mut self, f: fn(u8) -> bool) -> String {
        let position = self.position;
        while f(self.ch) {
            self.read_char();
        }
        self.input[position..self.position].to_string()
    }

    fn read_literal(&mut self) -> String {
        self.read_char();
        let position = self.position;
        while self.ch as char != '\"' {
            self.read_char();
        }
        self.input[position..self.position].to_string()
    }

    fn skip_line_comment(&mut self) {
        while self.ch != '\n' as u8 && self.ch != 0 {
            self.read_char();
        }
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            self.skip_whitespace();
            let start = self.position;
            let lineno = self.lineno;
            if self.read_position > self.input.len() {
                return self.token(TokenType::EOF, "".to_string(), lineno, start, start);
            }

            let current_char = (self.ch as char).to_string();
            if let Some(tok) = token::OPERATORS.get(&current_char) {
                if *tok == TokenType::DIV && self.peek_ahead() == Some('/' as u8) {
                    self.skip_line_comment();
                    self.read_char();
                    return self.token(TokenType::ILLEGAL, String::new(), lineno, start, self.position);
                }

                return match tok {
                    TokenType::ASSIGN => self.read_assign_or_eq(start, lineno),
                    TokenType::GT => self.read_gt_or_geq(start, lineno),
                    TokenType::LT => self.read_lt_or_leq(start, lineno),
                    TokenType::NOT => self.read_not_or_neq(start, lineno),
                    TokenType::DIV => {
                        self.read_char();
                        self.token(TokenType::DIV, "/".to_string(), lineno, start, self.position)
                    }
                    _ => {
                        let literal = current_char.clone();
                        self.read_char();
                        self.token(tok.clone(), literal, lineno, start, self.position)
                    }
                };
            }

            if current_char == "\"" {
                let str = Lexer::read_literal(self);
                self.read_char();
                return self.token(TokenType::STRING, str, lineno, start, self.position);
            }

            if let Some(tok) = token::DELIMITERS.get(&current_char) {
                self.read_char();
                return self.token(tok.clone(), current_char, lineno, start, self.position);
            }

            if Lexer::is_letter(self.ch) {
                let literal = Lexer::read_identifier(self, Lexer::is_letter);
                let tok = Token::check_keyword(&literal);
                return self.token(tok, literal, lineno, start, self.position);
            }

            if Lexer::is_number(self.ch) {
                let literal = Lexer::read_identifier(self, Lexer::is_number);
                return self.token(TokenType::NUM, literal, lineno, start, self.position);
            }

            panic!("illegal identifier");
        }
    }

    fn read_assign_or_eq(&mut self, start: usize, lineno: usize) -> Token {
        if self.peek_ahead() == Some('=' as u8) {
            self.read_char();
            self.read_char();
            self.token(TokenType::EQ, "==".to_string(), lineno, start, self.position)
        } else {
            self.read_char();
            self.token(TokenType::ASSIGN, "=".to_string(), lineno, start, self.position)
        }
    }

    fn read_gt_or_geq(&mut self, start: usize, lineno: usize) -> Token {
        if self.peek_ahead() == Some('=' as u8) {
            self.read_char();
            self.read_char();
            self.token(TokenType::GEQ, ">=".to_string(), lineno, start, self.position)
        } else {
            self.read_char();
            self.token(TokenType::GT, ">".to_string(), lineno, start, self.position)
        }
    }

    fn read_lt_or_leq(&mut self, start: usize, lineno: usize) -> Token {
        if self.peek_ahead() == Some('=' as u8) {
            self.read_char();
            self.read_char();
            self.token(TokenType::LEQ, "<=".to_string(), lineno, start, self.position)
        } else {
            self.read_char();
            self.token(TokenType::LT, "<".to_string(), lineno, start, self.position)
        }
    }

    fn read_not_or_neq(&mut self, start: usize, lineno: usize) -> Token {
        if self.peek_ahead() == Some('=' as u8) {
            self.read_char();
            self.read_char();
            self.token(TokenType::NEQ, "!=".to_string(), lineno, start, self.position)
        } else {
            self.read_char();
            self.token(TokenType::NOT, "!".to_string(), lineno, start, self.position)
        }
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