use std::{collections::HashMap, mem::transmute};

use crate::{
    bytecode::Opcode,
    chunk::{disassemble::disassemble_chunk, Chunk, Lineno},
    lexer::{self, Lexer},
    token::{self, Token, TokenType},
    value::Value,
};

use self::{parse_rule::RULES, precedence::Precedence};

mod parse_rule;
mod precedence;

use parse_rule::ParseRule;

trait Parsable {
    fn unary(&mut self, _: bool);

    fn binary(&mut self, _: bool);

    fn grouping(&mut self, _: bool);

    fn number(&mut self, _: bool);

    fn literal(&mut self, _: bool);

    fn string(&mut self, _: bool);

    fn variable(&mut self, _: bool);

    fn and(&mut self, _: bool);

    fn or(&mut self, _: bool);
    // fn apply_parse_fn(&mut self, parse_fn: ParseFn);
}
pub struct Parser<'a> {
    previous: Token,
    current: Token,
    lexer: Lexer,
    chunk: &'a mut Chunk,
    compiler: Compiler,
}

impl Parsable for Parser<'_> {
    fn unary(&mut self, _: bool) {
        let operator_type = self.previous.type_.clone();
        self.parse_precedence(Precedence::PrecUnary); // evaluate the operand
        match operator_type {
            TokenType::MINUS => self.emit_opcode(Opcode::OP_NEGATE),
            TokenType::NOT => self.emit_opcode(Opcode::OP_NOT),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, _: bool) {
        let operator_type = self.previous.type_.clone();
        let rule = parse_rule::ParseRule::get_rule(operator_type.clone());
        if (rule.precedence as usize) < 11
        /*variant count for Precedence; !todo - replace with variant_count::<Precedence>() once stabilized*/
        {
            let next_precedence = unsafe { transmute(rule.precedence as i8 + 1) };
            self.parse_precedence(next_precedence);
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

    fn and(&mut self, _: bool) {
        let jump = self.emit_jump(Opcode::OP_JUMP_IF_FALSE(0));
        self.emit_opcode(Opcode::OP_POP);
        self.parse_precedence(Precedence::PrecAnd);
        self.patch_jump(jump);
    }

    fn or(&mut self, _: bool) {
        let else_jump = self.emit_jump(Opcode::OP_JUMP_IF_FALSE(0));
        let end_jump = self.emit_jump(Opcode::OP_JUMP(0));

        self.patch_jump(else_jump);
        self.emit_opcode(Opcode::OP_POP);

        self.parse_precedence(Precedence::PrecOr);
        self.patch_jump(end_jump);
    }

    fn grouping(&mut self, _: bool) {
        self.expression(); // recursively evaluate the expression between the parenthesis
        self.consume(TokenType::RPAREN, "Expected )");
    }

    fn number(&mut self, _: bool) {
        let value = Value::NUMBER(self.previous.literal.parse::<f64>().unwrap());
        self.emit_constant(value);
    }

    fn literal(&mut self, _: bool) {
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

    fn string(&mut self, _: bool) {
        let lexeme = self.previous.literal.clone();
        self.emit_constant(Value::STR(lexeme));
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(can_assign);
    }
}

impl Parser<'_> {
    pub fn new(lexer: Lexer, chunk: &mut Chunk) -> Parser {
        let current = Token::new_def();
        let previous = Token::new_def();
        let compiler = Compiler::new();
        Parser {
            previous,
            current,
            lexer,
            chunk,
            compiler,
        }
    }
    /* ======================= plumbing ====================== */
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

    fn emit_jump(&mut self, op: Opcode) -> usize {
        self.emit_opcode(op);
        self.chunk.code.len() - 1
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - offset - 1;
        // 0  1  *2  3  4  5  *6
        // i1 i2 i3 i4 i5 i6  i7
        // println!("{:?}", self.chunk.code[offset].0);
        if let Opcode::OP_JUMP_IF_FALSE(ref mut x) = self.chunk.code[offset].0 {
            *x = jump;
        } else if let Opcode::OP_JUMP(ref mut x) = self.chunk.code[offset].0 {
            *x = jump;
        }
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

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = RULES[self.previous.type_.clone() as usize];
        let can_assign = precedence <= Precedence::PrecAssignment;
        if let Some(prefix_fn) = prefix_rule.prefix {
            prefix_fn(self, can_assign);
        } else {
            panic!("error unexpected token");
        }

        while precedence <= ParseRule::get_rule(self.current.clone().type_).precedence {
            self.advance();
            if let Some(infix_rule) = ParseRule::get_rule(self.previous.clone().type_).infix {
                infix_rule(self, can_assign);
            }
        }

        if can_assign && self.match_token(TokenType::ASSIGN) {
            panic!("Invalid Assignment target");
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
    /* ==================== statement ======================== */
    fn statement(&mut self) {
        if self.match_token(TokenType::PRINT) {
            self.print_statement();
        } else if self.match_token(TokenType::LBRACE) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else if self.match_token(TokenType::IF) {
            self.if_statement();
        } else if self.match_token(TokenType::WHILE) {
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SEMICOLON, "Expected ';' after value");
        self.emit_opcode(Opcode::OP_PRINT);
    }

    fn if_statement(&mut self) {
        self.consume(TokenType::LPAREN, "Expected '(' before expression");
        self.expression();
        self.consume(TokenType::RPAREN, "Expected ')' after expression");
        let then_jump = self.emit_jump(Opcode::OP_JUMP_IF_FALSE(0));
        self.emit_opcode(Opcode::OP_POP);
        self.statement();
        let else_jump = self.emit_jump(Opcode::OP_JUMP(0));
        self.patch_jump(then_jump);
        self.emit_opcode(Opcode::OP_POP);

        if self.match_token(TokenType::ELSE) {
            self.statement();
        }

        self.patch_jump(else_jump);
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
        let var_token = self.previous.clone();
        if self.match_token(TokenType::ASSIGN) {
            self.expression();
        } else {
            self.emit_opcode(Opcode::OP_NIL);
        }
        self.consume(
            TokenType::SEMICOLON,
            "Expected ';' after variable declaration",
        );
        self.define_variable(global_idx, &var_token);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PrecAssignment);
    }
    /* ==================== blocks =========================== */
    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        let mut popped = 0;
        for (_, scopes) in &mut self.compiler.locals {
            for i in (0..scopes.len()).rev() {
                if scopes[i].0 == self.compiler.scope_depth {
                    popped += 1;
                    self.compiler.total -= 1;
                    scopes.pop();
                }
            }
        }
        self.compiler.locals.retain(|_, v| v.len() > 0);
        self.compiler.scope_depth -= 1;
        for _ in 0..popped {
            self.emit_opcode(Opcode::OP_POP);
        }
    }

    fn block(&mut self) {
        while !self.check_token_type(TokenType::RBRACE) && !self.check_token_type(TokenType::EOF) {
            self.declaration();
        }

        self.consume(TokenType::RBRACE, "Expected } at end of block");
    }
    /* ==================== variable ========================= */
    fn identifier_constant(&mut self, token: Token) -> usize {
        self.chunk.add_constant(Value::STR(token.literal))
    }

    fn parse_variable(&mut self, err: &str) -> usize {
        self.consume(TokenType::IDENT, err);

        self.declare_variable();
        if self.compiler.scope_depth > 0 {
            return 0;
        }
        self.identifier_constant(self.previous.clone())
    }

    fn declare_variable(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
        if self.compiler.locals.contains_key(&self.previous)
            && self.compiler.locals[&self.previous].last().unwrap().0 == self.compiler.scope_depth
        {
            panic!(
                "Variable of name :{} already exists",
                &self.previous.literal
            );
        }
        self.add_local(self.previous.clone());
    }

    fn add_local(&mut self, token: Token) {
        if self.compiler.locals.contains_key(&token) {
            (*self.compiler.locals.get_mut(&token).unwrap()).push((-1, self.compiler.total));
        } else {
            let v = vec![(-1, self.compiler.total)];
            self.compiler.locals.insert(token, v);
        }
        self.compiler.total += 1;
    }

    fn mark_initialized(&mut self, var_token: &Token) {
        self.compiler.locals.get_mut(var_token).and_then(|v| {
            v.last_mut()
                .and_then(|i| Some(i.0 = self.compiler.scope_depth))
        });
    }

    fn define_variable(&mut self, idx: usize, var_token: &Token) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized(var_token);
            return;
        }
        self.emit_opcode(Opcode::OP_DEFINE_GLOBAL(idx));
    }

    fn resolve_local(&self, token: &Token) -> Option<i8> {
        if self.compiler.locals.contains_key(token) {
            let (depth, index) = self.compiler.locals[token].last().unwrap();
            if *depth == -1 {
                panic!("Cannot read local variable in its own initializer");
            } else {
                return Some(*index);
            }
        }
        None
    }

    fn named_variable(&mut self, can_assign: bool) {
        let get_op: Opcode;
        let set_op: Opcode;
        let slot = self.resolve_local(&self.previous);
        match slot {
            Some(slot_index) => {
                get_op = Opcode::OP_GET_LOCAL(slot_index.try_into().unwrap());
                set_op = Opcode::OP_SET_LOCAL(slot_index.try_into().unwrap());
            }
            _ => {
                let idx = self.identifier_constant(self.previous.clone());
                get_op = Opcode::OP_GET_GLOBAL(idx);
                set_op = Opcode::OP_SET_GLOBAL(idx);
            }
        }
        if can_assign && self.match_token(TokenType::ASSIGN) {
            self.expression();
            self.emit_opcode(set_op);
        } else {
            self.emit_opcode(get_op);
        }
    }
    fn end_compiler(&mut self) {
        self.emit_opcode(Opcode::OP_RETURN);
    }
}

#[derive(Eq, PartialEq, Hash)]
struct Local {
    name: Token,
    depth: i8,
}
struct Compiler {
    locals: HashMap<Token, Vec<(i8, i8)>>, // token -> (scope_depth, slot_idx)
    scope_depth: i8,
    total: i8,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            locals: HashMap::new(),
            scope_depth: 0,
            total: 0,
        }
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
