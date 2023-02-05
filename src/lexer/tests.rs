use crate::token::{Token, TokenType};
/************************** TESTS *******************/
#[cfg(test)]
use crate::token::{Ident, Keywords, Num, Operators};

use super::Lexer;

#[test]
fn test_token_next() {
    let input = "fn x = 10;";
    let mut lexer = super::Lexer::new(String::from(input));
    let rhs = lexer.next_token();
    let lhs = super::token::Token {
        type_: super::token::TokenType::KEYWORDS(Keywords::FUNCTION),
        literal: "fn".to_string(),
    };
    let rhs1 = lexer.next_token();
    let lhs1 = super::token::Token {
        type_: super::token::TokenType::IDENT(Ident::IDENT),
        literal: "x".to_string(),
    };
    let rhs2 = lexer.next_token();
    let lhs2 = super::token::Token {
        type_: super::token::TokenType::OPERATORS(Operators::ASSIGN),
        literal: "=".to_string(),
    };
    let rhs3 = lexer.next_token();
    let lhs3 = super::token::Token {
        type_: super::token::TokenType::NUM(Num::NUM),
        literal: "10".to_string(),
    };

    assert_eq!(lhs, rhs);
    assert_eq!(lhs1, rhs1);
    assert_eq!(lhs2, rhs2);
    assert_eq!(lhs3, rhs3);
}

#[test]
fn test_comments()
{
    let input = "//10\n10".to_string();
    let mut lexer = Lexer::new(input);
    lexer.next_token();
    let rhs = lexer.next_token();
    let lhs = Token {
        type_: TokenType::NUM(Num::NUM),
        literal: "10".to_string(),
    };

    assert_eq!(lhs, rhs);
}
