use std::mem;

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BinaryOp, BlockStmt, CallExpr, Decl, Expr, FnDecl, ForStmt, Ident,
        IfStmt, Literal, LogicalExpr, LogicalOp, Program, Stmt, UnaryExpr, UnaryOp, VarDecl,
        WhileStmt,
    },
    bytecode::Opcode,
    chunk::{Chunk, Lineno},
    gc::{Gc, GcRef},
    object::ObjFunction,
    value::Value,
};

use super::compiler::{CompilerState, FunctionType, Local, STACK_SIZE, CompileError};

pub struct Codegen<'a> {
    gc: &'a mut Gc,
    compiler: Box<CompilerState>,
    previous_line: usize,
}

impl<'a> Codegen<'a> {
    pub fn new_script(gc: &'a mut Gc) -> Self {
        let script_name = gc.intern("script".to_owned());
        Codegen {
            gc,
            compiler: CompilerState::new(script_name, FunctionType::Script),
            previous_line: 1,
        }
    }

    pub fn finish(mut self) -> Result<GcRef<ObjFunction>, CompileError> {
        self.emit_return();
        Ok(self.gc.alloc(self.compiler.function))
    }

    pub fn emit_program(&mut self, program: &Program) -> Result<(), CompileError> {
        for decl in &program.declarations {
            self.set_line(decl.span.line);
            self.emit_decl(&decl.node)?;
        }
        Ok(())
    }

    fn set_line(&mut self, line: usize) {
        self.previous_line = line;
    }

    fn chunk(&mut self) -> &mut Chunk {
        &mut self.compiler.function.chunk
    }

    fn emit_opcode(&mut self, op: Opcode) {
        self.compiler
            .function
            .chunk
            .write_chunk(op, Lineno(self.previous_line));
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

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk().code.len() - offset - 1;
        if let Opcode::OP_JUMP_IF_FALSE(ref mut x) = self.chunk().code[offset].0 {
            *x = jump;
        } else if let Opcode::OP_JUMP(ref mut x) = self.chunk().code[offset].0 {
            *x = jump;
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let idx = self.chunk().add_constant(value);
        self.emit_opcode(Opcode::OP_CONSTANT(idx));
    }

    fn emit_return(&mut self) {
        self.emit_opcode(Opcode::OP_NIL);
        self.emit_opcode(Opcode::OP_RETURN);
    }

    fn emit_decl(&mut self, decl: &Decl) -> Result<(), CompileError> {
        match decl {
            Decl::Var(var) => self.emit_var_decl(&var.node),
            Decl::Function(func) => self.emit_fn_decl(&func.node),
            Decl::Statement(stmt) => self.emit_stmt(&stmt.node),
        }
    }

    fn emit_stmt(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.emit_expr(&expr.node)?;
                self.emit_opcode(Opcode::OP_POP);
            }
            Stmt::Print(expr) => {
                self.emit_expr(&expr.node)?;
                self.emit_opcode(Opcode::OP_PRINT);
            }
            Stmt::Block(block) => {
                self.begin_scope();
                self.emit_block(&block.node)?;
                self.end_scope();
            }
            Stmt::If(if_stmt) => self.emit_if(&if_stmt.node)?,
            Stmt::While(while_stmt) => self.emit_while(&while_stmt.node)?,
            Stmt::For(for_stmt) => self.emit_for(&for_stmt.node)?,
            Stmt::Return(value) => self.emit_return_stmt(&value.node)?,
        }
        Ok(())
    }

    fn emit_block(&mut self, block: &BlockStmt) -> Result<(), CompileError> {
        for decl in &block.declarations {
            self.set_line(decl.span.line);
            self.emit_decl(&decl.node)?;
        }
        Ok(())
    }

    fn emit_if(&mut self, if_stmt: &IfStmt) -> Result<(), CompileError> {
        self.emit_expr(&if_stmt.condition)?;
        let then_jump = self.emit_jump(Opcode::OP_JUMP_IF_FALSE(0));
        self.emit_opcode(Opcode::OP_POP);
        self.emit_stmt(&if_stmt.then_branch)?;
        let else_jump = self.emit_jump(Opcode::OP_JUMP(0));
        self.patch_jump(then_jump);
        self.emit_opcode(Opcode::OP_POP);
        if let Some(else_branch) = &if_stmt.else_branch {
            self.emit_stmt(else_branch)?;
        }
        self.patch_jump(else_jump);
        Ok(())
    }

    fn emit_while(&mut self, while_stmt: &WhileStmt) -> Result<(), CompileError> {
        let loop_start = self.chunk().code.len();
        self.emit_expr(&while_stmt.condition)?;
        let exit_jump = self.emit_jump(Opcode::OP_JUMP_IF_FALSE(0));
        self.emit_opcode(Opcode::OP_POP);
        self.emit_stmt(&while_stmt.body)?;
        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);
        self.emit_opcode(Opcode::OP_POP);
        Ok(())
    }

    fn emit_for(&mut self, for_stmt: &ForStmt) -> Result<(), CompileError> {
        self.begin_scope();
        if let Some(init) = &for_stmt.initializer {
            self.emit_decl(init)?;
        }

        let mut loop_start = self.chunk().code.len();
        let mut exit_jump = None;
        if let Some(condition) = &for_stmt.condition {
            self.emit_expr(condition)?;
            exit_jump = Some(self.emit_jump(Opcode::OP_JUMP_IF_FALSE(0)));
            self.emit_opcode(Opcode::OP_POP);
        }

        if let Some(increment) = &for_stmt.increment {
            let body_jump = self.emit_jump(Opcode::OP_JUMP(0));
            let increment_start = self.chunk().code.len();
            self.emit_expr(increment)?;
            self.emit_opcode(Opcode::OP_POP);
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.emit_stmt(&for_stmt.body)?;
        self.emit_loop(loop_start);

        if let Some(jump) = exit_jump {
            self.patch_jump(jump);
            self.emit_opcode(Opcode::OP_POP);
        }

        self.end_scope();
        Ok(())
    }

    fn emit_return_stmt(&mut self, value: &Option<Expr>) -> Result<(), CompileError> {
        if matches!(self.compiler.f_type, FunctionType::Script) {
            return Err(CompileError("Cannot return from top-level code".to_string()));
        }
        if let Some(expr) = value {
            self.emit_expr(expr)?;
        } else {
            self.emit_opcode(Opcode::OP_NIL);
        }
        self.emit_opcode(Opcode::OP_RETURN);
        Ok(())
    }

    fn emit_var_decl(&mut self, var: &VarDecl) -> Result<(), CompileError> {
        let global_idx = self.declare_global(&var.name)?;
        if let Some(init) = &var.initializer {
            self.emit_expr(init)?;
        } else {
            self.emit_opcode(Opcode::OP_NIL);
        }
        self.define_variable_by_index(global_idx)
    }

    fn emit_fn_decl(&mut self, func: &FnDecl) -> Result<(), CompileError> {
        let global = self.declare_global(&func.name)?;
        self.mark_initialized();
        self.emit_function(func)?;
        self.define_variable_by_index(global)
    }

    fn emit_function(&mut self, func: &FnDecl) -> Result<(), CompileError> {
        self.push_compiler(&func.name)?;
        self.compiler.function.arity = func.params.len() as u8;
        self.begin_scope();

        for param in &func.params {
            self.declare_local(&param.name)?;
            self.define_local();
        }

        self.emit_block(&func.body)?;
        let function = self.end_compiler()?;
        let function = self.gc.alloc(function);
        self.emit_constant(Value::FUNCTION(function));
        Ok(())
    }

    fn emit_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Literal(lit) => {
                self.emit_literal(&lit.node);
            }
            Expr::Variable(var) => {
                self.emit_get_variable(&var.node.name, var.span.line)?;
            }
            Expr::Unary(unary) => self.emit_unary(&unary.node)?,
            Expr::Binary(binary) => self.emit_binary(&binary.node)?,
            Expr::Logical(logical) => self.emit_logical(&logical.node)?,
            Expr::Assign(assign) => self.emit_assign(&assign.node)?,
            Expr::Call(call) => self.emit_call(&call.node)?,
            Expr::Grouping(grouped) => self.emit_expr(&grouped.node)?,
        }
        Ok(())
    }

    fn emit_literal(&mut self, literal: &Literal) {
        match literal {
            Literal::Number(n) => self.emit_constant(Value::NUMBER(*n)),
            Literal::String(s) => {
                let interned = self.gc.intern(s.clone());
                self.emit_constant(Value::STR(interned));
            }
            Literal::Bool(true) => self.emit_opcode(Opcode::OP_TRUE),
            Literal::Bool(false) => self.emit_opcode(Opcode::OP_FALSE),
            Literal::Nil => self.emit_opcode(Opcode::OP_NIL),
        }
    }

    fn emit_unary(&mut self, unary: &UnaryExpr) -> Result<(), CompileError> {
        self.emit_expr(&unary.operand)?;
        match unary.op {
            UnaryOp::Negate => self.emit_opcode(Opcode::OP_NEGATE),
            UnaryOp::Not => self.emit_opcode(Opcode::OP_NOT),
        }
        Ok(())
    }

    fn emit_binary(&mut self, binary: &BinaryExpr) -> Result<(), CompileError> {
        self.emit_expr(&binary.left)?;
        self.emit_expr(&binary.right)?;
        match binary.op {
            BinaryOp::Add => self.emit_opcode(Opcode::OP_ADD),
            BinaryOp::Subtract => self.emit_opcode(Opcode::OP_SUBSTRACT),
            BinaryOp::Multiply => self.emit_opcode(Opcode::OP_MULTIPLY),
            BinaryOp::Divide => self.emit_opcode(Opcode::OP_DIVIDE),
            BinaryOp::Greater => self.emit_opcode(Opcode::OP_GT),
            BinaryOp::Less => self.emit_opcode(Opcode::OP_LT),
            BinaryOp::Equal => self.emit_opcode(Opcode::OP_EQ),
            BinaryOp::GreaterEqual => self.emit_opcodes(Opcode::OP_LT, Opcode::OP_NOT),
            BinaryOp::LessEqual => self.emit_opcodes(Opcode::OP_GT, Opcode::OP_NOT),
            BinaryOp::NotEqual => self.emit_opcodes(Opcode::OP_EQ, Opcode::OP_NOT),
        }
        Ok(())
    }

    fn emit_logical(&mut self, logical: &LogicalExpr) -> Result<(), CompileError> {
        self.emit_expr(&logical.left)?;
        match logical.op {
            LogicalOp::And => {
                let jump = self.emit_jump(Opcode::OP_JUMP_IF_FALSE(0));
                self.emit_opcode(Opcode::OP_POP);
                self.emit_expr(&logical.right)?;
                self.patch_jump(jump);
            }
            LogicalOp::Or => {
                let else_jump = self.emit_jump(Opcode::OP_JUMP_IF_FALSE(0));
                let end_jump = self.emit_jump(Opcode::OP_JUMP(0));
                self.patch_jump(else_jump);
                self.emit_opcode(Opcode::OP_POP);
                self.emit_expr(&logical.right)?;
                self.patch_jump(end_jump);
            }
        }
        Ok(())
    }

    fn emit_assign(&mut self, assign: &AssignExpr) -> Result<(), CompileError> {
        self.emit_expr(&assign.value)?;
        self.emit_set_variable(&assign.name.name, assign.name.span.line)
    }

    fn emit_call(&mut self, call: &CallExpr) -> Result<(), CompileError> {
        self.emit_expr(&call.callee)?;
        for arg in &call.args {
            self.emit_expr(arg)?;
        }
        self.emit_opcode(Opcode::OP_CALL(call.args.len() as u8));
        Ok(())
    }

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

    fn declare_global(&mut self, name: &Ident) -> Result<usize, CompileError> {
        if self.compiler.scope_depth > 0 {
            self.declare_local(&name.name)?;
            return Ok(0);
        }
        Ok(self.identifier_constant(&name.name))
    }

    fn define_variable(&mut self, name: &Ident) -> Result<(), CompileError> {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return Ok(());
        }
        let idx = self.identifier_constant(&name.name);
        self.emit_opcode(Opcode::OP_DEFINE_GLOBAL(idx));
        Ok(())
    }

    fn define_variable_by_index(&mut self, idx: usize) -> Result<(), CompileError> {
        if self.compiler.scope_depth > 0 {
            self.mark_initialized();
            return Ok(());
        }
        self.emit_opcode(Opcode::OP_DEFINE_GLOBAL(idx));
        Ok(())
    }

    fn declare_local(&mut self, name: &str) -> Result<(), CompileError> {
        if self.compiler.scope_depth == 0 {
            return Ok(());
        }
        for local in &self.compiler.locals[..self.compiler.total] {
            if local.depth != -1 && local.depth < self.compiler.scope_depth {
                break;
            }
            if local.name == name {
                return Err(CompileError(format!(
                    "Variable with name {} already exists",
                    name
                )));
            }
        }
        if self.compiler.total == STACK_SIZE {
            return Err(CompileError("Stack overflow; too many local variables".to_string()));
        }
        self.compiler.locals[self.compiler.total] = Local {
            name: name.to_string(),
            depth: -1,
        };
        self.compiler.total += 1;
        Ok(())
    }

    fn define_local(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
        self.mark_initialized();
    }

    fn mark_initialized(&mut self) {
        if self.compiler.scope_depth == 0 {
            return;
        }
        self.compiler.locals[self.compiler.total - 1].depth = self.compiler.scope_depth;
    }

    fn resolve_local(&self, name: &str) -> Result<Option<usize>, CompileError> {
        for (i, local) in self.compiler.locals.iter().enumerate().take(self.compiler.total) {
            if local.name == name {
                if local.depth == -1 {
                    return Err(CompileError(format!(
                        "Cannot read variable '{}' in its own initializer",
                        name
                    )));
                }
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    fn emit_get_variable(&mut self, name: &str, line: usize) -> Result<(), CompileError> {
        self.set_line(line);
        if let Some(slot) = self.resolve_local(name)? {
            self.emit_opcode(Opcode::OP_GET_LOCAL(slot));
        } else {
            let idx = self.identifier_constant(name);
            self.emit_opcode(Opcode::OP_GET_GLOBAL(idx));
        }
        Ok(())
    }

    fn emit_set_variable(&mut self, name: &str, line: usize) -> Result<(), CompileError> {
        self.set_line(line);
        if let Some(slot) = self.resolve_local(name)? {
            self.emit_opcode(Opcode::OP_SET_LOCAL(slot));
        } else {
            let idx = self.identifier_constant(name);
            self.emit_opcode(Opcode::OP_SET_GLOBAL(idx));
        }
        Ok(())
    }

    fn identifier_constant(&mut self, name: &str) -> usize {
        let identifier = self.gc.intern(name.to_owned());
        self.chunk().add_constant(Value::STR(identifier))
    }

    fn push_compiler(&mut self, name: &Ident) -> Result<(), CompileError> {
        let f_name = self.gc.intern(name.name.clone());
        let compiler = CompilerState::new(f_name, FunctionType::Function);
        let old_compiler = mem::replace(&mut self.compiler, compiler);
        self.compiler.enclosing = Some(old_compiler);
        Ok(())
    }

    fn end_compiler(&mut self) -> Result<ObjFunction, CompileError> {
        self.emit_return();
        if let Some(enclosing) = self.compiler.enclosing.take() {
            let compiler = mem::replace(&mut self.compiler, enclosing);
            Ok(compiler.function)
        } else {
            Err(CompileError("Enclosing compiler not found".to_string()))
        }
    }
}