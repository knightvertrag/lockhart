use crate::token;

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

    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while Self::is_letter(self.ch) {
            self.read_char();
        }
        self.input[position..self.position].to_string()
    }
    fn next_token(&mut self) -> token::Token {
        let res: token::Token;
        match self.ch as char {
            '=' => {
                res = token::Token::new(token::ASSIGN, self.ch.to_string());
            }
            '+' => {
                res = token::Token::new(token::PLUS, self.ch.to_string());
            }
            '(' => {
                res = token::Token::new(token::LPAREN, self.ch.to_string());
            }
            ')' => {
                res = token::Token::new(token::RPAREN, self.ch.to_string());
            }
            '{' => {
                res = token::Token::new(token::LBRACE, self.ch.to_string());
            }
            '}' => {
                res = token::Token::new(token::RBRACE, self.ch.to_string());
            }
            ';' => {
                res = token::Token::new(token::SEMICOLON, self.ch.to_string());
            }
            ',' => {
                res = token::Token::new(token::COMMA, self.ch.to_string());
            }
            '\0' => {
                res = token::Token::new(token::EOF, "".to_string());
            }
            _ => {
                if Self::is_letter(self.ch) {
                    res = token::Token::new(token::IDENT, Self::read_identifier(self));
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
        let input = "x = 10;";
        let mut lexer = super::Lexer::new(String::from(input));
        let rhs = lexer.next_token();
        let lhs = super::token::Token {
            type_: super::token::IDENT,
            literal: "x".to_string(),
        };
        assert_eq!(lhs, rhs);
    }
}
