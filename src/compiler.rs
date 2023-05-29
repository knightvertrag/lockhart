use std::{mem::transmute, rc::Rc, cell::RefCell};

use crate::{
    bytecode::Opcode,
    chunk::{Chunk, Lineno, disassemble::disassemble_code},
    lexer::Lexer,
    token::{Token, TokenType},
    value::Value,
};

use self::{parse_rule::RULES, precedence::Precedence};

mod parse_rule;
mod precedence;

use parse_rule::ParseRule;

pub struct Parser {
    previous: Token,
    current: Token,
    lexer: Lexer,
    chunk: Rc<RefCell<Chunk>>,
}

impl Parser {
    pub fn new(lexer: Lexer, chunk: Rc<RefCell<Chunk>>) -> Parser {
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

    fn emit_opcode(&mut self, opcode: Opcode) {
        // Rc::<Chunk>::get_mut(&mut self.chunk).unwrap().write_chunk(opcode, Lineno(self.previous.lineno));
        // self.chunk.borrow_mut().write_chunk(opcode, Lineno(self.previous.lineno));
        (*self.chunk).borrow_mut().write_chunk(opcode, Lineno(self.previous.lineno));

        // self.chunk.get_mut().write_chunk(opcode, Lineno(self.previous.lineno));
    }
    /// check if the current token is expected token and advance the lexer
    fn consume(&mut self, type_: TokenType, err: &str) {
        if self.current.type_ == type_ {
            self.advance();
        } else {
            panic!("{}", err); // panic if current token in not the expected token
        }
    }
    
    fn emit_constant(&mut self, value: Value) {
        let idx = (*self.chunk).borrow_mut().add_constant(value);
        self.emit_opcode(Opcode::OPCONSTANT(idx));
    }

    fn parse_precendence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = RULES[self.current.type_.clone() as usize];
        if let Some(prefix_fn) = prefix_rule.prefix {
            prefix_fn(self);
        }
        // else {
        //     panic!("error unexpected token");
        // }

        while precedence <= ParseRule::get_rule(self.current.clone().type_).precedence {
            self.advance();
            let infix_rule_option = ParseRule::get_rule(self.previous.clone().type_).infix;
            if let Some(infix_rule) = infix_rule_option {
                infix_rule(self);
            } else {
                panic!("error unexpected token");
            }
        }
    }
    fn number(&mut self) {
        let value = Value::NUMBER(str::parse::<f64>(&self.current.literal).unwrap());
        self.emit_constant(value);
    }

    fn grouping(&mut self) {
        self.expression(); // recursively evaluate the expression between the parenthesis
        self.consume(TokenType::RPAREN, "Expected )"); // consume the closing paren or throw error
    }

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
        let rule = parse_rule::ParseRule::get_rule(operator_type);
        if (rule.precedence as i8) < 11 {
            let next_precedence: Precedence = unsafe { transmute(rule.precedence as i8 + 1) };
            self.parse_precendence(next_precedence);
        }

        match self.previous.type_ {
            TokenType::PLUS => self.emit_opcode(Opcode::OPADD),
            TokenType::MINUS => self.emit_opcode(Opcode::OPSUBSTRACT),
            TokenType::MUL => self.emit_opcode(Opcode::OPMULTIPLY),
            TokenType::DIV => self.emit_opcode(Opcode::OPDIVIDE),
            _ => {
                return;
            }
        }
    }
    fn expression(&mut self) {
        self.parse_precendence(Precedence::PrecAssignment);
    }
}

pub fn compile(source: String, chunk: Chunk) -> Result<(), &'static str> {
    let lexer = Lexer::new(source);
    let chunk_shared = Rc::new(RefCell::new(chunk));
    let mut parser = Parser::new(lexer, chunk_shared.clone());
    // parser.consume(TokenType::EOF, "Expected EOF");
    // parser.advance();
    parser.expression();
    disassemble_code(chunk_shared.clone(), "TEST");
    Ok(())
}
