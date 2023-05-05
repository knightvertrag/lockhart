use crate::token::{Token, TokenType};
/************************** TESTS *******************/
#[cfg(test)]
use super::Lexer;

#[test]
fn test_token_next() {
    let input = "fn x = 10;";
    let mut lexer = super::Lexer::new(String::from(input));
    let rhs = lexer.next_token();
    let lhs = super::token::Token {
        type_: super::token::TokenType::FUNCTION,
        literal: "fn".to_string(),
        lineno: 1,
    };
    let rhs1 = lexer.next_token();
    let lhs1 = super::token::Token {
        type_: super::token::TokenType::IDENT,
        literal: "x".to_string(),
        lineno: 1,
    };
    let rhs2 = lexer.next_token();
    let lhs2 = super::token::Token {
        type_: super::token::TokenType::ASSIGN,
        literal: "=".to_string(),
        lineno: 1,
    };
    let rhs3 = lexer.next_token();
    let lhs3 = super::token::Token {
        type_: super::token::TokenType::NUM,
        literal: "10".to_string(),
        lineno: 1,
    };

    assert_eq!(lhs, rhs);
    assert_eq!(lhs1, rhs1);
    assert_eq!(lhs2, rhs2);
    assert_eq!(lhs3, rhs3);
}

#[test]
fn test_comments() {
    let input = "//10\n10".to_string();
    let mut lexer = Lexer::new(input);
    lexer.next_token();
    let rhs = lexer.next_token();
    let lhs = Token {
        type_: TokenType::NUM,
        literal: "10".to_string(),
        lineno: 2,
    };

    assert_eq!(lhs, rhs);
}
#[test]
fn test_error() {
    let input = "1str".to_string();
    let mut lexer = Lexer::new(input);
    let rh1 = lexer.next_token();
    println!("{}", rh1.literal);
}
