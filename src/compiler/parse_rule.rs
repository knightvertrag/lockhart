use crate::token::{TokenType::{self, *}};

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
type ParseFn = fn(&mut Parser) -> ();
#[derive(Clone, Copy)]
pub struct ParseRule {
    pub prefix: Option<ParseFn>,
    pub infix: Option<ParseFn>,
    pub precedence: Precedence,
}

impl ParseRule {
    pub fn get_rule(token_type: TokenType) -> &'static ParseRule {
        &RULES[token_type as usize]
    }
}

pub static RULES: [ParseRule; 30] = {
    let mut a = [ParseRule {
        prefix: None,
        infix: None,
        precedence: PrecNone,
    }; 30];
    rule!(a, IDENT, None, None, PrecNone);
    rule!(a, NUM, Some(Parser::number), None, PrecNone);
    rule!(a, LITERAL, None, None, PrecNone);
    rule!(a, LET, None, None, PrecNone);
    rule!(a, FUNCTION, None, None, PrecNone);
    rule!(a, IF, None, None, PrecNone);
    rule!(a, ELSE, None, None, PrecNone);
    rule!(a, RETURN, None, None, PrecNone);
    rule!(a, TRUE, None, None, PrecNone);
    rule!(a, FALSE, None, None, PrecNone);
    rule!(a, ASSIGN, None, None, PrecNone);
    rule!(a, NOT, None, None, PrecNone);
    rule!(a, GT, None, None, PrecNone);
    rule!(a, LT, None, None, PrecNone);
    rule!(a, GEQ, None, None, PrecNone);
    rule!(a, LEQ, None, None, PrecNone);
    rule!(a, EQ, None, None, PrecNone);
    rule!(a, NEQ, None, None, PrecNone);
    rule!(a, PLUS, None, Some(Parser::binary), PrecTerm);
    rule!(a, MINUS, Some(Parser::unary), Some(Parser::binary), PrecTerm);
    rule!(a, MUL, None, Some(Parser::binary), PrecFactor);
    rule!(a, DIV, None, Some(Parser::binary), PrecFactor);
    rule!(a, COMMA, None, None, PrecNone);
    rule!(a, SEMICOLON, None, None, PrecNone);
    rule!(a, LBRACE, Some(Parser::grouping), None, PrecNone);
    rule!(a, RBRACE, Some(Parser::grouping), None, PrecNone);
    rule!(a, LPAREN, Some(Parser::grouping), None, PrecNone);
    rule!(a, RPAREN, Some(Parser::grouping), None, PrecNone);
    rule!(a, ILLEGAL, None, None, PrecNone);
    rule!(a, EOF, None, None, PrecNone);

    a
};
