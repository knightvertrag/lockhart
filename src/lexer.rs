use crate::token::{self, Token};

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

    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while Self::is_letter(self.ch) {
            self.read_char();
        }
        self.input[position..self.position].to_string()
    }
    fn next_token(&mut self) -> token::Token {
        self.skip_whitespace();
        let res: token::Token;
        match self.ch as char {
            '=' => {
                res = token::Token::new(token::ASSIGN, token::ASSIGN.to_string());
            }
            '+' => {
                res = token::Token::new(token::PLUS, token::PLUS.to_string());
            }
            '(' => {
                res = token::Token::new(token::LPAREN, token::LPAREN.to_string());
            }
            ')' => {
                res = token::Token::new(token::RPAREN, token::RPAREN.to_string());
            }
            '{' => {
                res = token::Token::new(token::LBRACE, token::LBRACE.to_string());
            }
            '}' => {
                res = token::Token::new(token::RBRACE, token::RBRACE.to_string());
            }
            ';' => {
                res = token::Token::new(token::SEMICOLON, token::SEMICOLON.to_string());
            }
            ',' => {
                res = token::Token::new(token::COMMA, token::COMMA.to_string());
            }
            '\0' => {
                res = token::Token::new(token::EOF, "\0".to_string());
            }
            _ => {
                if Self::is_letter(self.ch) {
                    let literal = Lexer::read_identifier(self);
                    let tok = token::Token::check_keyword(&literal);
                    res = token::Token::new(tok, literal);
                    return res;
                } else {
                    res = token::Token::new(token::ILLEGAL, self.ch.to_string());
                }
            }
        }
        self.read_char();
        res
    }
}

/************************** TESTS *******************/
#[cfg(test)]
mod tests {
    #[test]
    fn test_token_next() {
        let input = "fn x = 10;";
        let mut lexer = super::Lexer::new(String::from(input));
        let rhs = lexer.next_token();
        let lhs = super::token::Token {
            type_: super::token::FUNCTION,
            literal: "fn".to_string(),
        };
        let rhs1 = lexer.next_token();
        let lhs1 = super::token::Token {
            type_: super::token::IDENT,
            literal: "x".to_string(),
        };
        let rhs2 = lexer.next_token();
        let lhs2 = super::token::Token {
            type_: super::token::ASSIGN,
            literal: "=".to_string(),
        };
        assert_eq!(lhs, rhs);
        assert_eq!(lhs1, rhs1);
        assert_eq!(lhs2, rhs2);
    }
}
