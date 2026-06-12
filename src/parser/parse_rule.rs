use crate::token::TokenType::{self, *};

use super::precedence::Precedence;
use super::precedence::Precedence::*;
use super::Parser;

macro_rules! rule {
    ($a: ident, $tok: ident, $prefix: expr, $infix: expr, $precedence: expr) => {
        $a[$tok as usize] = ParseRule {
            prefix: $prefix,
            infix: $infix,
            precedence: $precedence,
        }
    };
}

pub type PrefixFn = fn(&mut Parser, bool) -> Result<crate::ast::Expr, crate::parser::ParseError>;
pub type InfixFn =
    fn(&mut Parser, bool, crate::ast::Expr) -> Result<crate::ast::Expr, crate::parser::ParseError>;

#[derive(Clone, Copy)]
pub struct ParseRule {
    pub prefix: Option<PrefixFn>,
    pub infix: Option<InfixFn>,
    pub precedence: Precedence,
}

impl ParseRule {
    pub fn get_rule(token_type: TokenType) -> ParseRule {
        RULES[token_type as usize]
    }
}

pub static RULES: [ParseRule; 36] = {
    let mut a = [ParseRule {
        prefix: None,
        infix: None,
        precedence: PrecNone,
    }; 36];
    rule!(a, IDENT, Some(Parser::parse_variable_expr), None, PrecNone);
    rule!(a, NUM, Some(Parser::parse_number_expr), None, PrecNone);
    rule!(a, STRING, Some(Parser::parse_string_expr), None, PrecNone);
    rule!(a, LET, None, None, PrecNone);
    rule!(a, FUNCTION, None, None, PrecNone);
    rule!(a, PRINT, None, None, PrecNone);
    rule!(a, IF, None, None, PrecNone);
    rule!(a, ELSE, None, None, PrecNone);
    rule!(a, FOR, None, None, PrecNone);
    rule!(a, WHILE, None, None, PrecNone);
    rule!(a, RETURN, None, None, PrecNone);
    rule!(a, TRUE, Some(Parser::parse_literal_expr), None, PrecNone);
    rule!(a, FALSE, Some(Parser::parse_literal_expr), None, PrecNone);
    rule!(a, NIL, Some(Parser::parse_literal_expr), None, PrecNone);
    rule!(a, ASSIGN, None, None, PrecNone);
    rule!(a, NOT, Some(Parser::parse_unary_expr), None, PrecNone);
    rule!(a, GT, None, Some(Parser::parse_binary_expr), PrecComparison);
    rule!(a, LT, None, Some(Parser::parse_binary_expr), PrecComparison);
    rule!(
        a,
        GEQ,
        None,
        Some(Parser::parse_binary_expr),
        PrecComparison
    );
    rule!(
        a,
        LEQ,
        None,
        Some(Parser::parse_binary_expr),
        PrecComparison
    );
    rule!(a, EQ, None, Some(Parser::parse_binary_expr), PrecEquality);
    rule!(a, NEQ, None, Some(Parser::parse_binary_expr), PrecEquality);
    rule!(a, PLUS, None, Some(Parser::parse_binary_expr), PrecTerm);
    rule!(
        a,
        MINUS,
        Some(Parser::parse_unary_expr),
        Some(Parser::parse_binary_expr),
        PrecTerm
    );
    rule!(a, MUL, None, Some(Parser::parse_binary_expr), PrecFactor);
    rule!(a, DIV, None, Some(Parser::parse_binary_expr), PrecFactor);
    rule!(a, AND, None, Some(Parser::parse_logical_expr), PrecAnd);
    rule!(a, OR, None, Some(Parser::parse_logical_expr), PrecOr);
    rule!(a, COMMA, None, None, PrecNone);
    rule!(a, SEMICOLON, None, None, PrecNone);
    rule!(a, LBRACE, None, None, PrecNone);
    rule!(a, RBRACE, None, None, PrecNone);
    rule!(
        a,
        LPAREN,
        Some(Parser::parse_grouping_expr),
        Some(Parser::parse_call_expr),
        PrecCall
    );
    rule!(a, RPAREN, None, None, PrecNone);
    rule!(a, ILLEGAL, None, None, PrecNone);
    rule!(a, EOF, None, None, PrecNone);

    a
};
