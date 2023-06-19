use std::{cell::RefCell, fmt::Error, mem::transmute, rc::Rc};

use crate::{
    bytecode::Opcode,
    chunk::{disassemble::disassemble_chunk, Chunk, Lineno},
    lexer::{self, Lexer},
    token::{Token, TokenType},
    value::Value,
};

use self::{parse_rule::RULES, precedence::Precedence};

mod parse_rule;
mod precedence;

use parse_rule::ParseRule;

// !todo - Switch to idiomatic enum matching for dispactching correct parse function
// #[derive(Copy, Clone)]
// pub enum ParseFn {
//     Unary,
//     Binary,
//     Grouping,
//     Number,
// }

// struct Parse_Rule {
//     infix: Option<ParseFn>,
//     prefix: Option<ParseFn>,
//     precedence: Precedence,
// }

// impl Parse_Rule {
//     fn new(infix: Option<ParseFn>, prefix: Option<ParseFn>, precedence: Precedence) -> Parse_Rule {
//         Parse_Rule {
//             infix,
//             prefix,
//             precedence,
//         }
//     }
//     fn get_rule(token_type: TokenType) -> Parse_Rule {
//         use {Precedence::*, TokenType::*};
//         match token_type {
//             NUM => Parse_Rule::new(Some(ParseFn::Number), None, PrecNone),
//             PLUS => Parse_Rule::new(Some(ParseFn::Binary), None, PrecTerm),
//             MINUS => Parse_Rule::new(Some(ParseFn::Binary), None, PrecTerm),
//             DIV => Parse_Rule::new(Some(ParseFn::Binary), None, PrecTerm),

//             _ => Parse_Rule::new(None, None, Precedence::PrecNone),
//         }
//     }
// }

trait Parsable {
    fn unary(&mut self);

    fn binary(&mut self);

    fn grouping(&mut self);

    fn number(&mut self);

    fn literal(&mut self);

    fn string(&mut self);

    fn variable(&mut self);

    // fn apply_parse_fn(&mut self, parse_fn: ParseFn);
}
pub struct Parser<'a> {
    previous: Token,
    current: Token,
    lexer: Lexer,
    chunk: &'a mut Chunk,
}

impl Parsable for Parser<'_> {
    fn unary(&mut self) {
        let operator_type = self.previous.type_.clone();
        self.parse_precendence(Precedence::PrecUnary); // evaluate the operand
        match operator_type {
            TokenType::MINUS => self.emit_opcode(Opcode::OP_NEGATE),
            TokenType::NOT => self.emit_opcode(Opcode::OP_NOT),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
        let operator_type = self.previous.type_.clone();
        let rule = parse_rule::ParseRule::get_rule(operator_type.clone());
        if (rule.precedence as usize) < 11
        /*variant count for Precedence; !todo - replace with variant_count::<Precedence>() once stabilized*/
        {
            let next_precedence = unsafe { transmute(rule.precedence as i8 + 1) };
            self.parse_precendence(next_precedence);
        }

        match operator_type {
            TokenType::PLUS => self.emit_opcode(Opcode::OP_ADD),
            TokenType::MINUS => self.emit_opcode(Opcode::OP_SUBSTRACT),
            TokenType::MUL => self.emit_opcode(Opcode::OP_MULTIPLY),
            TokenType::DIV => self.emit_opcode(Opcode::OP_DIVIDE),
            TokenType::GT => self.emit_opcode(Opcode::OP_GT),
            TokenType::LT => self.emit_opcode(Opcode::OP_LT),
            TokenType::EQ => self.emit_opcode(Opcode::OP_EQ),
            // todo: use dedicated opcodes and implementations for double operators
            TokenType::GEQ => self.emit_opcodes(Opcode::OP_LT, Opcode::OP_NOT),
            TokenType::LEQ => self.emit_opcodes(Opcode::OP_GT, Opcode::OP_NOT),
            TokenType::NEQ => self.emit_opcodes(Opcode::OP_EQ, Opcode::OP_NOT),
            _ => unreachable!(),
        }
    }

    fn grouping(&mut self) {
        self.expression(); // recursively evaluate the expression between the parenthesis
        self.consume(TokenType::RPAREN, "Expected )");
    }

    fn number(&mut self) {
        let value = Value::NUMBER(self.previous.literal.parse::<f64>().unwrap());
        self.emit_constant(value);
    }

    fn literal(&mut self) {
        match self.previous.type_ {
            TokenType::TRUE => {
                self.emit_opcode(Opcode::OP_TRUE);
            }
            TokenType::FALSE => {
                self.emit_opcode(Opcode::OP_FALSE);
            }
            TokenType::NIL => {
                self.emit_opcode(Opcode::OP_NIL);
            }
            _ => unreachable!(),
        }
    }

    fn string(&mut self) {
        let lexeme = self.previous.literal.clone();
        self.emit_constant(Value::STR(lexeme));
    }

    fn variable(&mut self) {
        self.named_variable();
    }
    // fn apply_parse_fn(&mut self, parse_fn: ParseFn) {
    //     match parse_fn {
    //         ParseFn::Binary => self.binary(),
    //         ParseFn::Unary => self.unary(),
    //         ParseFn::Grouping => self.grouping(),
    //         ParseFn::Number => self.number(),
    //     }
    // }
}

pub trait Statement {
    fn statement(&mut self);

    fn print_statement(&mut self);

    fn expression_statement(&mut self);

    fn declaration(&mut self);

    fn variable_declaration(&mut self);
}

impl Statement for Parser<'_> {
    fn statement(&mut self) {
        if self.match_token(TokenType::PRINT) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SEMICOLON, "Expected ';' after value");
        self.emit_opcode(Opcode::OP_PRINT);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SEMICOLON, "Expected ; after expression");
        self.emit_opcode(Opcode::OP_POP);
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::LET) {
            self.variable_declaration();
        } else {
            self.statement();
        }
    }

    fn variable_declaration(&mut self) {
        let global_idx = self.parse_variable("Expected variable name");
        if self.match_token(TokenType::ASSIGN) {
            self.expression();
        } else {
            self.emit_opcode(Opcode::OP_NIL);
        }
        self.consume(
            TokenType::SEMICOLON,
            "Expected ';' after variable declaration",
        );
        self.define_variable(global_idx);
    }
}
trait Expression {
    fn expression(&mut self);
}

impl Expression for Parser<'_> {
    fn expression(&mut self) {
        self.parse_precendence(Precedence::PrecAssignment);
    }
}
impl Parser<'_> {
    pub fn new(lexer: Lexer, chunk: &mut Chunk) -> Parser {
        let current = Token::new_def();
        let previous = Token::new_def();
        Parser {
            previous,
            current,
            lexer,
            chunk,
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        self.current = self.lexer.next_token();
    }

    fn emit_opcode(&mut self, op: Opcode) {
        self.chunk.write_chunk(op, Lineno(self.previous.lineno));
    }

    fn emit_opcodes(&mut self, op1: Opcode, op2: Opcode) {
        self.emit_opcode(op1);
        self.emit_opcode(op2);
    }

    fn consume(&mut self, type_: TokenType, err: &str) {
        if self.current.type_ == type_ {
            self.advance();
        } else {
            panic!("{}", err); // panic if current token in not the expected token
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let idx = self.chunk.add_constant(value);
        self.emit_opcode(Opcode::OP_CONSTANT(idx));
    }

    fn parse_precendence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = RULES[self.previous.type_.clone() as usize];
        if let Some(prefix_fn) = prefix_rule.prefix {
            prefix_fn(self);
        } else {
            panic!("error unexpected token");
        }

        while precedence <= ParseRule::get_rule(self.current.clone().type_).precedence {
            self.advance();
            let infix_rule_option = ParseRule::get_rule(self.previous.clone().type_).infix;
            if let Some(infix_rule) = infix_rule_option {
                infix_rule(self);
            }
        }
    }

    /* ====================== utils ========================== */
    #[inline(always)]
    fn check_token_type(&self, type_: TokenType) -> bool {
        self.current.type_ == type_
    }

    fn match_token(&mut self, type_: TokenType) -> bool {
        if self.check_token_type(type_) {
            self.advance();
            true
        } else {
            false
        }
    }
    /* ==================== variable ========================= */
    fn identifier_constant(&mut self, token: Token) -> usize {
        self.chunk.add_constant(Value::STR(token.literal))
    }

    fn parse_variable(&mut self, err: &str) -> usize {
        self.consume(TokenType::IDENT, err);
        self.identifier_constant(self.previous.clone())
    }

    fn define_variable(&mut self, idx: usize) {
        self.emit_opcode(Opcode::OP_DEFINE_GLOBAL(idx));
    }

    fn named_variable(&mut self) {
        let idx = self.identifier_constant(self.previous.clone());
        self.emit_opcode(Opcode::OP_GET_GLOBAL(idx));
    }
    fn end_compiler(&mut self) {
        self.emit_opcode(Opcode::OP_RETURN);
    }
}

pub fn compile(source: String, chunk: &mut Chunk) -> Result<(), &'static str> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer, chunk);
    parser.advance();

    while !parser.match_token(TokenType::EOF) {
        parser.declaration();
    }
    // parser.expression();
    // parser.consume(TokenType::EOF, "Expected EOF");
    parser.end_compiler();
    // disassemble_chunk(chunk, "TEST");
    Ok(())
}
