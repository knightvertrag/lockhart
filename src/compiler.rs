use std::mem::{self, transmute};

use crate::{
    bytecode::Opcode,
    chunk::{disassemble::disassemble_chunk, Chunk, Lineno},
    gc::{Gc, GcRef},
    lexer::Lexer,
    object::{ObjFunction, ObjString},
    token::{Token, TokenType},
    value::Value,
    vm::InterpretError,
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

    fn call(&mut self, _: bool);
    // fn apply_parse_fn(&mut self, parse_fn: Parse Fn);
}
pub struct Parser<'a> {
    previous: Token,
    current: Token,
    lexer: Lexer,
    gc: &'a mut Gc,
    compiler: Box<Compiler>,
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
        let operator_type = self.previous.type_;
        let rule = parse_rule::ParseRule::get_rule(operator_type);
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
        let interned_s = self.gc.intern(lexeme);
        self.emit_constant(Value::STR(interned_s));
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(can_assign);
    }

    fn call(&mut self, _: bool) {
        let count = self.arg_count();
        self.emit_opcode(Opcode::OP_CALL(count));
    }
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer, gc: &'a mut Gc) -> Parser<'a> {
        let current = Token::new_def();
        let previous = Token::new_def();
        let function_name = gc.intern("script".to_owned());
        let compiler = Compiler::new(function_name, FunctionType::SCRIPT);
        Parser {
            previous,
            current,
            lexer,
            gc,
            compiler,
        }
    }
    /* ======================= plumbing ====================== */
    fn advance(&mut self) {
        self.previous = self.current.clone();
        self.current = self.lexer.next_token();
    }

    fn chunk(&mut self) -> &mut Chunk {
        &mut self.compiler.function.chunk
    }

    fn emit_opcode(&mut self, op: Opcode) {
        self.compiler
            .function
            .chunk
            .write_chunk(op, Lineno(self.previous.lineno));
    }

    fn emit_opcodes(&mut self, op1: Opcode, op2: Opcode) {
        self.emit_opcode(op1);
        self.emit_opcode(op2);
    }

    fn emit_jump(&mut self, op: Opcode) -> usize {
        self.emit_opcode(op);
        self.chunk().code.len() - 1
    }

    fn emit_loop(&mut self, loop_start: usize) {
        let jump = self.chunk().code.len() - loop_start + 1;
        self.emit_opcode(Opcode::OP_LOOP(jump));
    }

    fn emit_return(&mut self) {
        // match self.compiler.f_type {
        //     _ => self.emit_opcode(Opcode::OP_NIL)
        // }
        self.emit_opcode(Opcode::OP_NIL);
        self.emit_opcode(Opcode::OP_RETURN);
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk().code.len() - offset - 1;
        // 0  1  *2  3  4  5  *6
        // i1 i2 i3 i4 i5 i6  i7
        // println!("{:?}", self.chunk().code[offset].0);
        if let Opcode::OP_JUMP_IF_FALSE(ref mut x) = self.chunk().code[offset].0 {
            *x = jump;
        } else if let Opcode::OP_JUMP(ref mut x) = self.chunk().code[offset].0 {
            *x = jump;
        }
    }

    fn consume(&mut self, type_: TokenType, err: &str) {
        if self.current.type_ == type_ {
            self.advance();
        } else {
            panic!("{}", err);
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let idx = self.chunk().add_constant(value);
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
        } else if self.match_token(TokenType::RETURN) {
            self.return_statement();
        } else if self.match_token(TokenType::WHILE) {
            self.while_statement();
        } else if self.match_token(TokenType::FOR) {
            self.for_statement();
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

    fn while_statement(&mut self) {
        let loop_start = self.chunk().code.len();
        self.consume(TokenType::LPAREN, "Expected '(' after while");
        self.expression();
        self.consume(TokenType::RPAREN, "Expected ')' after condition");
        let jump = self.emit_jump(Opcode::OP_JUMP_IF_FALSE(0));
        self.emit_opcode(Opcode::OP_POP);
        self.statement();
        self.emit_loop(loop_start);
        self.patch_jump(jump);
        self.emit_opcode(Opcode::OP_POP);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(TokenType::LPAREN, "Expected '(' after for");
        // initializer clause
        if self.match_token(TokenType::SEMICOLON) {
            // no initializer
        } else if self.match_token(TokenType::LET) {
            self.variable_declaration();
        } else {
            self.expression_statement();
        }

        // condition clause
        let mut loop_start = self.chunk().code.len();
        let mut exit_jump = None;
        if !self.match_token(TokenType::SEMICOLON) {
            self.expression();
            self.consume(TokenType::SEMICOLON, "Expected ';' after loop condition");
            exit_jump = Some(self.emit_jump(Opcode::OP_JUMP_IF_FALSE(0)));
            self.emit_opcode(Opcode::OP_POP);
        }

        // incrememt clause
        if !self.match_token(TokenType::RPAREN) {
            let body_jump = self.emit_jump(Opcode::OP_JUMP(0));
            let increment_start = self.chunk().code.len();
            self.expression();
            self.emit_opcode(Opcode::OP_POP);
            self.consume(TokenType::RPAREN, "Expected ')' after for clause");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }
        self.statement();
        self.emit_loop(loop_start);

        if let Some(x) = exit_jump {
            self.patch_jump(x);
            self.emit_opcode(Opcode::OP_POP);
        }
        self.end_scope();
    }

    fn return_statement(&mut self) {
        if let FunctionType::SCRIPT = self.compiler.f_type {
            panic!("Cannot return from top-level code");
        }
        if self.match_token(TokenType::SEMICOLON) {
            self.emit_return();
        } else {
            self.expression();
            self.consume(TokenType::SEMICOLON, "Expected ; after return statement");
            self.emit_opcode(Opcode::OP_RETURN);
        }
    }
    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::SEMICOLON, "Expected ; after expression");
        self.emit_opcode(Opcode::OP_POP);
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::FUNCTION) {
            self.function_declaration();
        } else if self.match_token(TokenType::LET) {
            self.variable_declaration();
        } else {
            self.statement();
        }
    }

    fn function_declaration(&mut self) {
        let global = self.parse_variable("Expected Function name");
        self.mark_initialized();
        self.function(FunctionType::FUNCTION);
        self.define_variable(global);
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

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PrecAssignment);
    }
    /* ==================== blocks =========================== */
    fn begin_scope(&mut self) {
        self.compiler.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.compiler.scope_depth -= 1;
        while self.compiler.total > 0
            && self.compiler.locals[self.compiler.total - 1].depth > self.compiler.scope_depth
        {
            self.emit_opcode(Opcode::OP_POP);
            self.compiler.total -= 1;
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
        let identifier = self.gc.intern(token.literal);
        self.compiler
            .function
            .chunk
            .add_constant(Value::STR(identifier))
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
        let var_token = &self.previous;
        for local in &self.compiler.locals {
            if local.depth != -1 && local.depth < self.compiler.scope_depth {
                break;
            }
            if local.name.literal == var_token.literal {
                panic!(
                    "Error: Variable with name {} already exists",
                    var_token.literal
                );
            }
        }
        self.add_local(self.previous.clone());
    }

    fn add_local(&mut self, token: Token) {
        if self.compiler.total == STACK_SIZE {
            panic!("Stack overflow; too many local variables");
        }
        self.compiler.locals[self.compiler.total] = Local {
            name: token,
            depth: -1,
        };
        self.compiler.total += 1;
    }

    fn mark_initialized(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
        self.compiler.locals[self.compiler.total - 1].depth = self.compiler.scope_depth;
    }

    fn define_variable(&mut self, idx: usize) {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_opcode(Opcode::OP_DEFINE_GLOBAL(idx));
    }

    fn arg_count(&mut self) -> u8 {
        let mut count: u8 = 0;
        if !self.check_token_type(TokenType::RPAREN) {
            loop {
                self.expression();
                if count == u8::MAX {
                    panic!("Cannot have more than {} arguments", count);
                }
                count += 1;
                if !self.match_token(TokenType::COMMA) {
                    break;
                }
            }
        }
        self.consume(TokenType::RPAREN, "Expected ')' after arguments.");
        count
    }
    fn resolve_local(&self, token: &Token) -> Option<i8> {
        for (i, local) in self.compiler.locals.iter().enumerate() {
            if local.name.literal == token.literal {
                if local.depth == -1 {
                    panic!("Cannot read variable into its own initializer");
                }
                return Some(i as i8);
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

    fn function(&mut self, f_type: FunctionType) {
        self.push_compiler(f_type);
        self.begin_scope();
        self.consume(TokenType::LPAREN, "Expected '(' after function name");
        if !self.check_token_type(TokenType::RPAREN) {
            loop {
                self.compiler.function.arity += 1;
                if self.compiler.function.arity > u8::MAX {
                    panic!("Cannot have more than {} parameters", u8::MAX);
                }
                let constant = self.parse_variable("Expected parameter name");
                self.define_variable(constant);
                if !self.match_token(TokenType::COMMA) {
                    break;
                }
            }
        }
        self.consume(TokenType::RPAREN, "Expected ')' after parameters");
        self.consume(TokenType::LBRACE, "Expected '{' before function body");
        self.block();
        let function = self.end_compiler();
        let function = self.gc.alloc(function);
        self.emit_constant(Value::FUNCTION(function));
    }

    fn push_compiler(&mut self, f_type: FunctionType) {
        let f_name = self.gc.intern(self.previous.literal.clone());
        let compiler = Compiler::new(f_name, f_type);
        let old_compiler = mem::replace(&mut self.compiler, compiler);
        self.compiler.enclosing = Some(old_compiler);
    }

    fn end_compiler(&mut self) -> ObjFunction {
        self.emit_return();
        if let Some(enclosing) = self.compiler.enclosing.take() {
            let compiler = mem::replace(&mut self.compiler, enclosing);
            return compiler.function;
        } else {
            panic!("Enclosing compiler not found");
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
struct Local {
    name: Token,
    depth: i8,
}

const STACK_SIZE: usize = 50000;
struct Compiler {
    enclosing: Option<Box<Compiler>>,
    function: ObjFunction,
    f_type: FunctionType,
    locals: Vec<Local>,
    scope_depth: i8,
    total: usize,
}

enum FunctionType {
    FUNCTION,
    SCRIPT,
}

impl Compiler {
    fn new(function_name: GcRef<ObjString>, f_type: FunctionType) -> Box<Compiler> {
        let function = ObjFunction::new(function_name);
        let array_repeat_value: Local = Local {
            name: Token::new_def(),
            depth: -1,
        };
        let compiler = Compiler {
            enclosing: None,
            function,
            f_type,
            locals: vec![array_repeat_value; STACK_SIZE],
            scope_depth: 0,
            total: 1, //0th slot for vm internal use
        };

        Box::new(compiler)
    }
}

pub fn compile(source: String, gc: &mut Gc) -> Result<GcRef<ObjFunction>, InterpretError> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer, gc);
    parser.advance();

    while !parser.match_token(TokenType::EOF) {
        parser.declaration();
    }
    // let function = parser.end_compiler();
    // disassemble_chunk(chunk, "TEST");
    parser.emit_return();
    Ok(parser.gc.alloc(parser.compiler.function))
}
