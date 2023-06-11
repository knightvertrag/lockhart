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
            if self.ch == '\n' as u8 {
                self.lineno += 1;
            }
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

    fn read_literal(&mut self) -> String {
        self.read_char();
        let position = self.position;
        while self.ch as char != '\"' {
            self.read_char();
        }
        self.input[position..self.position].to_string()
    }
    pub fn next_token(&mut self) -> Token {
        if self.read_position > self.input.len() {
            return Token::new(TokenType::EOF, "".to_string(), self.lineno)
        }
        self.skip_whitespace();
        let mut token = Token::new(TokenType::ILLEGAL, "".to_string(), self.lineno);

        let current_char = (self.ch as char).to_string();
        if let Some(tok) = token::OPERATORS.get(&current_char) {
            let mut build_double = |t, next_ch: char, lit: &str| {
                if self.peek_ahead() == Some(next_ch as u8) {
                    let mut literal = lit.to_string();
                    literal.push(next_ch);
                    token = Token::new(t, literal, self.lineno);
                    self.read_char();
                } else {
                    token = Token::new(tok.clone(), lit.to_string(), self.lineno);
                }
            };

            match tok {
                TokenType::ASSIGN => {
                    build_double(TokenType::EQ, '=', "=");
                }
                TokenType::GT => {
                    build_double(TokenType::GEQ, '=', ">");
                }
                TokenType::LT => {
                    build_double(TokenType::LEQ, '=', "<");
                }
                TokenType::NOT => {
                    build_double(TokenType::NEQ, '=', "!");
                }
                TokenType::DIV => {
                    // Check for comment
                    if self.peek_ahead() == Some('/' as u8) {
                        while self.ch != '\n' as u8 {
                            self.read_char();
                        }
                    } else {
                        token = Token::new(tok.clone(), current_char, self.lineno);
                    }
                }
                _ => {
                    token = Token::new(tok.clone(), current_char, self.lineno);
                }
            }
        } else if current_char == "\"" {
            // string literal
            let str = Lexer::read_literal(self);
            token = Token::new(TokenType::STRING, str, self.lineno)
        } else if let Some(tok) = token::DELIMITERS.get(&current_char) {
            // delimiter
            token = Token::new(tok.clone(), current_char, self.lineno);
        } else {
            if Lexer::is_letter(self.ch) {
                // identifier
                let literal = Lexer::read_identifier(self, Lexer::is_letter);
                let tok = Token::check_keyword(&literal);
                token = Token::new(tok, literal, self.lineno);
                return token;
            } else if Lexer::is_number(self.ch) {
                // number literal
                let literal = Lexer::read_identifier(self, Lexer::is_number);
                token = Token::new(TokenType::NUM, literal, self.lineno);
                return token;
            } else {
                panic!("illegal identifier");
            }
        }

        self.read_char();
        token
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
