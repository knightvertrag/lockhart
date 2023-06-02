use std::{cell::RefCell, fmt::Error, mem::transmute, rc::Rc};

use crate::{
    bytecode::Opcode,
    chunk::{disassemble::disassemble_code, Chunk, Lineno},
    lexer::{self, Lexer},
    token::{Token, TokenType},
    value::Value,
};

use self::{parse_rule::RULES, precedence::Precedence};

mod parse_rule;
mod precedence;

use parse_rule::ParseRule;

#[derive(Copy, Clone)]
pub enum ParseFn {
    Unary,
    Binary,
    Grouping,
    Number,
}

struct Parse_Rule {
    infix: Option<ParseFn>,
    prefix: Option<ParseFn>,
    precedence: Precedence,
}

impl Parse_Rule {
    fn new(infix: Option<ParseFn>, prefix: Option<ParseFn>, precedence: Precedence) -> Parse_Rule {
        Parse_Rule {
            infix,
            prefix,
            precedence,
        }
    }
    fn get_rule(token_type: TokenType) -> Parse_Rule {
        use {Precedence::*, TokenType::*};
        match token_type {
            NUM => Parse_Rule::new(Some(ParseFn::Number), None, PrecNone),
            PLUS => Parse_Rule::new(Some(ParseFn::Binary), None, PrecTerm),
            MINUS => Parse_Rule::new(Some(ParseFn::Binary), None, PrecTerm),
            DIV => Parse_Rule::new(Some(ParseFn::Binary), None, PrecTerm),

            _ => Parse_Rule::new(None, None, Precedence::PrecNone),
        }
    }
}

pub trait Parsable {
    fn unary(&mut self);

    fn binary(&mut self);

    fn grouping(&mut self);

    fn number(&mut self);

    fn apply_parse_fn(&mut self, parse_fn: ParseFn);
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
        if let TokenType::MINUS = operator_type {
            self.emit_opcode(Opcode::OPNEGATE);
        } else {
            return;
        }
    }

    fn binary(&mut self) {
        let operator_type = self.previous.type_.clone();
        let rule = parse_rule::ParseRule::get_rule(operator_type.clone());
        if (rule.precedence as i8) < 11 {
            let next_precedence = unsafe { transmute(rule.precedence as i8 + 1) };
            self.parse_precendence(next_precedence);
        }

        match operator_type {
            TokenType::PLUS => self.emit_opcode(Opcode::OPADD),
            TokenType::MINUS => self.emit_opcode(Opcode::OPSUBSTRACT),
            TokenType::MUL => self.emit_opcode(Opcode::OPMULTIPLY),
            TokenType::DIV => self.emit_opcode(Opcode::OPDIVIDE),
            _ => {
                return;
            }
        }
    }

    fn grouping(&mut self) {
        self.expression(); // recursively evaluate the expression between the parenthesis
        self.consume(TokenType::RPAREN, "Expected )");
    }

    fn number(&mut self) {
        let value = Value::NUMBER(str::parse::<f64>(&self.previous.literal).unwrap());
        self.emit_constant(value);
    }

    fn apply_parse_fn(&mut self, parse_fn: ParseFn) {
        match parse_fn {
            ParseFn::Binary => self.binary(),
            ParseFn::Unary => self.unary(),
            ParseFn::Grouping => self.grouping(),
            ParseFn::Number => self.number(),
        }
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
        self.emit_opcode(Opcode::OPCONSTANT(idx));
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

    // fn number(&mut self) {}

    // fn grouping(&mut self) {}

    // fn unary(&mut self) {}

    // fn binary(&mut self) {}

    fn expression(&mut self) {
        self.parse_precendence(Precedence::PrecAssignment);
    }

    fn end_compiler(&mut self) {
        self.emit_opcode(Opcode::OPRETURN);
    }
}

pub fn compile(source: String, chunk: &mut Chunk) -> Result<(), &'static str> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer, chunk);
    parser.advance();
    parser.expression();
    parser.consume(TokenType::EOF, "Expected EOF");
    parser.end_compiler();
    // disassemble_code(chunk, "TEST");
    Ok(())
}
