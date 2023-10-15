use crate::{
    //compiler::{binary, grouping, number, unary},
    token::TokenType::{self, *},
    compiler::Parsable
};

use super::{precedence::Precedence, precedence::Precedence::*, Parser};

macro_rules! rule {
    ($a: ident, $tok: ident, $prefix: expr, $infix: expr, $precedence: expr) => {
        $a[$tok as usize] = ParseRule {
            prefix: $prefix,
            infix: $infix,
            precedence: $precedence,
        }
    };
}
type ParseFn = fn(&mut Parser, bool);
#[derive(Clone, Copy)]
pub struct ParseRule {
    pub prefix: Option<ParseFn>,
    pub infix: Option<ParseFn>,
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
    rule!(a, IDENT, Some(|x, y| x.variable(y)), None, PrecNone);
    rule!(a, NUM, Some(|x, y| x.number(y)), None, PrecNone);
    rule!(a, STRING, Some(|x, y| x.string(y)), None, PrecNone);
    rule!(a, LET, None, None, PrecNone);
    rule!(a, FUNCTION, None, None, PrecNone);
    rule!(a, PRINT, None, None, PrecNone);
    rule!(a, IF, None, None, PrecNone);
    rule!(a, ELSE, None, None, PrecNone);
    rule!(a, FOR, None, None, PrecNone);
    rule!(a, WHILE, None, None, PrecNone);
    rule!(a, RETURN, None, None, PrecNone);
    rule!(a, TRUE, Some(|x, y| x.literal(y)), None, PrecNone);
    rule!(a, FALSE, Some(|x, y| x.literal(y)), None, PrecNone);
    rule!(a, NIL, Some(|x, y| x.literal(y)), None, PrecNone);
    rule!(a, ASSIGN, None, None, PrecNone);
    rule!(a, NOT, Some(|x, y| x.unary(y)), None, PrecNone);
    rule!(a, GT, None, Some(|x, y| x.binary(y)), PrecComparison);
    rule!(a, LT,  None, Some(|x, y| x.binary(y)), PrecComparison);
    rule!(a, GEQ,  None, Some(|x, y| x.binary(y)), PrecComparison);
    rule!(a, LEQ,  None, Some(|x, y| x.binary(y)), PrecComparison);
    rule!(a, EQ,  None, Some(|x, y| x.binary(y)), PrecEquality);
    rule!(a, NEQ,  None, Some(|x, y| x.binary(y)), PrecEquality);
    rule!(a, PLUS, None, Some(|x, y| x.binary(y)), PrecTerm);
    rule!(a, MINUS, Some(|x, y| x.unary(y)), Some(|x, y| x.binary(y)), PrecTerm);
    rule!(a, MUL, None, Some(|x, y| x.binary(y)), PrecFactor);
    rule!(a, DIV, None, Some(|x, y| x.binary(y)), PrecFactor);
    rule!(a, AND, None, Some(|x, y| x.and(y)), PrecAnd);
    rule!(a, OR, None, Some(|x, y| x.or(y)), PrecOr);
    rule!(a, COMMA, None, None, PrecNone);
    rule!(a, SEMICOLON, None, None, PrecNone);
    rule!(a, LBRACE, Some(|x, y| x.grouping(y)), None, PrecNone);
    rule!(a, RBRACE, Some(|x, y| x.grouping(y)), None, PrecNone);
    rule!(a, LPAREN, Some(|x, y| x.grouping(y)), None, PrecNone);
    rule!(a, RPAREN, Some(|x, y| x.grouping(y)), None, PrecNone);
    rule!(a, ILLEGAL, None, None, PrecNone);
    rule!(a, EOF, None, None, PrecNone);

    a
};

