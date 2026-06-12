use crate::token::{Token, TokenType};
/************************** TESTS *******************/
#[cfg(test)]
use super::Lexer;

#[test]
fn test_token_next() {
    let input = "fn x = 10;";
    let mut lexer = super::Lexer::new(String::from(input));
    let rhs = lexer.next_token();
    let lhs = Token {
        type_: TokenType::FUNCTION,
        literal: "fn".to_string(),
        lineno: 1,
        start: 0,
        end: 2,
    };
    let rhs1 = lexer.next_token();
    let lhs1 = Token {
        type_: TokenType::IDENT,
        literal: "x".to_string(),
        lineno: 1,
        start: 3,
        end: 4,
    };
    let rhs2 = lexer.next_token();
    let lhs2 = Token {
        type_: TokenType::ASSIGN,
        literal: "=".to_string(),
        lineno: 1,
        start: 5,
        end: 6,
    };
    let rhs3 = lexer.next_token();
    let lhs3 = Token {
        type_: TokenType::NUM,
        literal: "10".to_string(),
        lineno: 1,
        start: 7,
        end: 9,
    };

    assert_eq!(lhs, rhs);
    assert_eq!(lhs1, rhs1);
    assert_eq!(lhs2, rhs2);
    assert_eq!(lhs3, rhs3);
}

#[test]
fn multiline_token_lineno() {
    let input = "fn a() {\n  return 1;\n}\n\nprint 1;".to_string();
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token.type_ == TokenType::EOF {
            break;
        }
        if token.type_ == TokenType::ILLEGAL {
            continue;
        }
        tokens.push((token.literal, token.lineno));
    }

    assert_eq!(lineno_of(&tokens, "fn"), 1);
    assert_eq!(lineno_of(&tokens, "return"), 2);
    assert_eq!(lineno_of(&tokens, "print"), 5);
}

fn lineno_of(tokens: &[(String, usize)], literal: &str) -> usize {
    tokens
        .iter()
        .find(|(lit, _)| lit == literal)
        .map(|(_, line)| *line)
        .unwrap_or_else(|| panic!("missing token '{literal}'"))
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
        start: 5,
        end: 7,
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