use std::{num::ParseFloatError, thread::panicking};

use crate::{
    bytecode::Opcode,
    chunk::{Chunk, Lineno},
    lexer::Lexer,
    token::{Delimiters, Token, TokenType, Operators},
    value::Value,
};

struct Parser<'a> {
    previous: Token,
    current: Token,
    lexer: Lexer,
    chunk: &'a mut Chunk,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer, chunk: &'a mut Chunk) -> Parser {
        let current = Token {
            literal: "".to_string(),
            type_: TokenType::EOF,
            lineno: 0,
        };
        let previous = current.clone();
        Parser {
            previous,
            current,
            lexer,
            chunk,
        }
    }
    fn advance(&mut self) {
        self.previous = self.current.clone();
        for token in &mut self.lexer {
            self.current = token;
        }
    }

    fn emit_opcode(&mut self, opcode: Opcode) {
        self.chunk.write_chunk(opcode, Lineno(self.previous.lineno));
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
        let idx = self.chunk.add_constant(value);
        self.emit_opcode(Opcode::OPCONSTANT(idx));
    }
    fn number(&mut self) -> Result<(), ParseFloatError> {
        let value = Value::NUMBER(str::parse::<f64>(&self.previous.literal)?);
        self.emit_constant(value);
        Ok(())
    }

    fn grouping(&mut self) {
        self.expression(); // recursively evaluate the expression between the parenthesis
        self.consume(TokenType::DELIMITERS(Delimiters::RPAREN), "Expected )"); // consume the closing paren or throw error
    }

    fn unary_negation(&mut self) {
        let operator = self.previous.type_.clone();
        self.expression();
        match operator {
            TokenType::OPERATORS(Operators::MINUS) => {
                self.emit_opcode(Opcode::OPNEGATE);
            }
            _ => ()
        }
    }
    fn expression(&mut self) {}
}

pub fn compile(source: String, mut chunk: Chunk) -> Result<(), &'static str> {
    let lexer = Lexer::new(source);
    let parser = Parser::new(lexer, &mut chunk);
    Ok(())
}
