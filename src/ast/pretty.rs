use std::fmt::Write;

use serde::Serialize;
use serde_json::{json, Value};

use super::{
    decl::{Decl, FnDecl, VarDecl},
    expr::{Expr, Literal},
    span::{Span, Spanned},
    stmt::{BlockStmt, ForStmt, Stmt},
    Program,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DumpFormat {
    Tree,
    Json,
}

pub fn dump_program(program: &Program, format: DumpFormat) -> String {
    match format {
        DumpFormat::Tree => print_program(program),
        DumpFormat::Json => program_to_json(program)
            .and_then(|v| serde_json::to_string_pretty(&v))
            .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}")),
    }
}

pub fn print_program(program: &Program) -> String {
    let mut out = String::new();
    let _ = print_program_to(&mut out, program);
    out
}

pub fn print_program_to<W: Write>(writer: &mut W, program: &Program) -> std::fmt::Result {
    let mut printer = TreePrinter::new(writer);
    printer.write_program(program)
}

struct TreePrinter<'a, W: Write> {
    writer: &'a mut W,
    indent: usize,
}

impl<'a, W: Write> TreePrinter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        TreePrinter { writer, indent: 0 }
    }

    fn write_program(&mut self, program: &Program) -> std::fmt::Result {
        writeln!(self.writer, "Program")?;
        for (i, decl) in program.declarations.iter().enumerate() {
            let last = i == program.declarations.len() - 1;
            self.write_decl(decl, last)?;
        }
        Ok(())
    }

    fn write_line(&mut self, prefix: &str, last: bool, content: &str) -> std::fmt::Result {
        let branch = if last { "└─ " } else { "├─ " };
        writeln!(
            self.writer,
            "{}{}{}",
            "  ".repeat(self.indent),
            if self.indent == 0 && prefix.is_empty() {
                branch.to_string()
            } else if prefix.is_empty() {
                branch.to_string()
            } else {
                format!("{branch}{prefix}")
            },
            content
        )
    }

    fn with_indent<F>(&mut self, f: F) -> std::fmt::Result
    where
        F: FnOnce(&mut Self) -> std::fmt::Result,
    {
        self.indent += 1;
        let result = f(self);
        self.indent -= 1;
        result
    }

    fn write_decl(&mut self, decl: &Spanned<Decl>, last: bool) -> std::fmt::Result {
        match &decl.node {
            Decl::Var(var) => {
                self.write_line("", last, &format!("Decl::Var [line {}]", decl.span.line))?;
                self.with_indent(|p| p.write_var_decl(&var.node))?;
            }
            Decl::Function(func) => {
                self.write_line(
                    "",
                    last,
                    &format!("Decl::Function [line {}]", decl.span.line),
                )?;
                self.with_indent(|p| p.write_fn_decl(&func.node))?;
            }
            Decl::Statement(stmt) => {
                self.write_line(
                    "",
                    last,
                    &format!("Decl::Statement [line {}]", decl.span.line),
                )?;
                self.with_indent(|p| p.write_stmt(&stmt.node, true))?;
            }
        }
        Ok(())
    }

    fn write_var_decl(&mut self, var: &VarDecl) -> std::fmt::Result {
        self.write_line("", false, &format!("name: {}", var.name.name))?;
        if let Some(init) = &var.initializer {
            self.write_line("", false, "initializer:")?;
            self.with_indent(|p| p.write_expr(init, true))?;
            self.write_line("", true, "")?;
        } else {
            self.write_line("", true, "initializer: nil")?;
        }
        Ok(())
    }

    fn write_fn_decl(&mut self, func: &FnDecl) -> std::fmt::Result {
        let params: Vec<_> = func.params.iter().map(|p| p.name.as_str()).collect();
        self.write_line("", false, &format!("name: {}", func.name.name))?;
        self.write_line("", false, &format!("params: [{}]", params.join(", ")))?;
        self.write_line("", true, "body:")?;
        self.with_indent(|p| p.write_block(&func.body, true))
    }

    fn write_block(&mut self, block: &BlockStmt, last: bool) -> std::fmt::Result {
        self.write_line("", last, "Block")?;
        self.with_indent(|p| {
            for (i, decl) in block.declarations.iter().enumerate() {
                let is_last = i == block.declarations.len() - 1;
                p.write_decl(decl, is_last)?;
            }
            Ok(())
        })
    }

    fn write_stmt(&mut self, stmt: &Stmt, last: bool) -> std::fmt::Result {
        match stmt {
            Stmt::Expression(expr) => {
                self.write_line(
                    "",
                    last,
                    &format!("Stmt::Expression [line {}]", expr.span.line),
                )?;
                self.with_indent(|p| p.write_expr(&expr.node, true))
            }
            Stmt::Print(expr) => {
                self.write_line("", last, &format!("Stmt::Print [line {}]", expr.span.line))?;
                self.with_indent(|p| p.write_expr(&expr.node, true))
            }
            Stmt::Block(block) => {
                self.write_line("", last, &format!("Stmt::Block [line {}]", block.span.line))?;
                self.with_indent(|p| p.write_block(&block.node, true))
            }
            Stmt::If(if_stmt) => {
                self.write_line("", last, &format!("Stmt::If [line {}]", if_stmt.span.line))?;
                self.with_indent(|p| {
                    p.write_line("", false, "condition:")?;
                    p.with_indent(|p| p.write_expr(&if_stmt.node.condition, true))?;
                    p.write_line("", false, "then:")?;
                    p.with_indent(|p| p.write_stmt(&if_stmt.node.then_branch, true))?;
                    if let Some(else_branch) = &if_stmt.node.else_branch {
                        p.write_line("", false, "else:")?;
                        p.with_indent(|p| p.write_stmt(else_branch, true))?;
                    }
                    p.write_line("", true, "")?;
                    Ok(())
                })
            }
            Stmt::While(while_stmt) => {
                self.write_line(
                    "",
                    last,
                    &format!("Stmt::While [line {}]", while_stmt.span.line),
                )?;
                self.with_indent(|p| {
                    p.write_line("", false, "condition:")?;
                    p.with_indent(|p| p.write_expr(&while_stmt.node.condition, true))?;
                    p.write_line("", true, "body:")?;
                    p.with_indent(|p| p.write_stmt(&while_stmt.node.body, true))
                })
            }
            Stmt::For(for_stmt) => {
                self.write_line("", last, &format!("Stmt::For [line {}]", for_stmt.span.line))?;
                self.with_indent(|p| p.write_for(&for_stmt.node))
            }
            Stmt::Return(value) => {
                self.write_line(
                    "",
                    last,
                    &format!("Stmt::Return [line {}]", value.span.line),
                )?;
                self.with_indent(|p| match &value.node {
                    Some(expr) => p.write_expr(expr, true),
                    None => p.write_line("", true, "nil"),
                })
            }
        }
    }

    fn write_for(&mut self, for_stmt: &ForStmt) -> std::fmt::Result {
        if let Some(init) = &for_stmt.initializer {
            self.write_line("", false, "initializer:")?;
            self.with_indent(|p| p.write_decl_as_root(init))?;
        } else {
            self.write_line("", false, "initializer: (none)")?;
        }
        if let Some(cond) = &for_stmt.condition {
            self.write_line("", false, "condition:")?;
            self.with_indent(|p| p.write_expr(cond, true))?;
        } else {
            self.write_line("", false, "condition: (none)")?;
        }
        if let Some(inc) = &for_stmt.increment {
            self.write_line("", false, "increment:")?;
            self.with_indent(|p| p.write_expr(inc, true))?;
        } else {
            self.write_line("", false, "increment: (none)")?;
        }
        self.write_line("", true, "body:")?;
        self.with_indent(|p| p.write_stmt(&for_stmt.body, true))
    }

    fn write_decl_as_root(&mut self, decl: &Decl) -> std::fmt::Result {
        match decl {
            Decl::Var(var) => self.write_var_decl(&var.node),
            Decl::Function(func) => self.write_fn_decl(&func.node),
            Decl::Statement(stmt) => self.write_stmt(&stmt.node, true),
        }
    }

    fn write_expr(&mut self, expr: &Expr, last: bool) -> std::fmt::Result {
        match expr {
            Expr::Literal(lit) => {
                self.write_line(
                    "",
                    last,
                    &format!("Literal({}) [line {}]", format_literal(&lit.node), lit.span.line),
                )
            }
            Expr::Variable(var) => self.write_line(
                "",
                last,
                &format!("Variable({}) [line {}]", var.node.name, var.span.line),
            ),
            Expr::Unary(unary) => {
                self.write_line(
                    "",
                    last,
                    &format!("Unary::{:?} [line {}]", unary.node.op, unary.span.line),
                )?;
                self.with_indent(|p| p.write_expr(&unary.node.operand, true))
            }
            Expr::Binary(binary) => {
                self.write_line(
                    "",
                    last,
                    &format!("Binary::{:?} [line {}]", binary.node.op, binary.span.line),
                )?;
                self.with_indent(|p| {
                    p.write_line("", false, "left:")?;
                    p.with_indent(|p| p.write_expr(&binary.node.left, true))?;
                    p.write_line("", true, "right:")?;
                    p.with_indent(|p| p.write_expr(&binary.node.right, true))
                })
            }
            Expr::Logical(logical) => {
                self.write_line(
                    "",
                    last,
                    &format!("Logical::{:?} [line {}]", logical.node.op, logical.span.line),
                )?;
                self.with_indent(|p| {
                    p.write_line("", false, "left:")?;
                    p.with_indent(|p| p.write_expr(&logical.node.left, true))?;
                    p.write_line("", true, "right:")?;
                    p.with_indent(|p| p.write_expr(&logical.node.right, true))
                })
            }
            Expr::Assign(assign) => {
                self.write_line(
                    "",
                    last,
                    &format!("Assign({}) [line {}]", assign.node.name.name, assign.span.line),
                )?;
                self.with_indent(|p| p.write_expr(&assign.node.value, true))
            }
            Expr::Call(call) => {
                self.write_line("", last, &format!("Call [line {}]", call.span.line))?;
                self.with_indent(|p| {
                    p.write_line("", false, "callee:")?;
                    p.with_indent(|p| p.write_expr(&call.node.callee, true))?;
                    for (i, arg) in call.node.args.iter().enumerate() {
                        let is_last = i == call.node.args.len() - 1;
                        p.write_line("", false, &format!("arg[{i}]:"))?;
                        p.with_indent(|p| p.write_expr(arg, is_last))?;
                    }
                    if call.node.args.is_empty() {
                        p.write_line("", true, "args: []")?;
                    } else {
                        p.write_line("", true, "")?;
                    }
                    Ok(())
                })
            }
            Expr::Grouping(grouped) => {
                self.write_line("", last, &format!("Grouping [line {}]", grouped.span.line))?;
                self.with_indent(|p| p.write_expr(&grouped.node, true))
            }
        }
    }
}

fn format_literal(literal: &Literal) -> String {
    match literal {
        Literal::Number(n) => n.to_string(),
        Literal::String(s) => format!("\"{s}\""),
        Literal::Bool(b) => b.to_string(),
        Literal::Nil => "nil".to_string(),
    }
}

#[derive(Serialize)]
struct JsonSpan {
    line: usize,
    start: usize,
    end: usize,
}

impl From<Span> for JsonSpan {
    fn from(span: Span) -> Self {
        JsonSpan {
            line: span.line,
            start: span.start,
            end: span.end,
        }
    }
}

fn program_to_json(program: &Program) -> serde_json::Result<Value> {
    Ok(json!({
        "type": "Program",
        "declarations": program
            .declarations
            .iter()
            .map(decl_to_json)
            .collect::<Vec<_>>(),
    }))
}

fn decl_to_json(decl: &Spanned<Decl>) -> Value {
    let span = JsonSpan::from(decl.span);
    match &decl.node {
        Decl::Var(var) => json!({
            "type": "Var",
            "span": span,
            "name": var.node.name.name,
            "initializer": var.node.initializer.as_ref().map(expr_to_json),
        }),
        Decl::Function(func) => json!({
            "type": "Function",
            "span": span,
            "name": func.node.name.name,
            "params": func.node.params.iter().map(|p| &p.name).collect::<Vec<_>>(),
            "body": block_to_json(&func.node.body),
        }),
        Decl::Statement(stmt) => json!({
            "type": "Statement",
            "span": span,
            "statement": stmt_to_json(&stmt.node),
        }),
    }
}

fn block_to_json(block: &BlockStmt) -> Value {
    json!({
        "type": "Block",
        "declarations": block
            .declarations
            .iter()
            .map(decl_to_json)
            .collect::<Vec<_>>(),
    })
}

fn stmt_to_json(stmt: &Stmt) -> Value {
    match stmt {
        Stmt::Expression(expr) => json!({
            "type": "Expression",
            "span": JsonSpan::from(expr.span),
            "expression": expr_to_json(&expr.node),
        }),
        Stmt::Print(expr) => json!({
            "type": "Print",
            "span": JsonSpan::from(expr.span),
            "expression": expr_to_json(&expr.node),
        }),
        Stmt::Block(block) => {
            let mut value = block_to_json(&block.node);
            if let Some(obj) = value.as_object_mut() {
                obj.insert("span".to_string(), json!(JsonSpan::from(block.span)));
            }
            value
        }
        Stmt::If(if_stmt) => json!({
            "type": "If",
            "span": JsonSpan::from(if_stmt.span),
            "condition": expr_to_json(&if_stmt.node.condition),
            "then": stmt_to_json(&if_stmt.node.then_branch),
            "else": if_stmt.node.else_branch.as_ref().map(|s| stmt_to_json(s)),
        }),
        Stmt::While(while_stmt) => json!({
            "type": "While",
            "span": JsonSpan::from(while_stmt.span),
            "condition": expr_to_json(&while_stmt.node.condition),
            "body": stmt_to_json(&while_stmt.node.body),
        }),
        Stmt::For(for_stmt) => json!({
            "type": "For",
            "span": JsonSpan::from(for_stmt.span),
            "initializer": for_stmt
                .node
                .initializer
                .as_ref()
                .map(|decl| decl_to_json_unspanned(decl)),
            "condition": for_stmt.node.condition.as_ref().map(expr_to_json),
            "increment": for_stmt.node.increment.as_ref().map(expr_to_json),
            "body": stmt_to_json(&for_stmt.node.body),
        }),
        Stmt::Return(value) => json!({
            "type": "Return",
            "span": JsonSpan::from(value.span),
            "value": value.node.as_ref().map(expr_to_json),
        }),
    }
}

fn decl_to_json_unspanned(decl: &Decl) -> Value {
    match decl {
        Decl::Var(var) => json!({
            "type": "Var",
            "name": var.node.name.name,
            "initializer": var.node.initializer.as_ref().map(expr_to_json),
        }),
        Decl::Function(func) => json!({
            "type": "Function",
            "name": func.node.name.name,
            "params": func.node.params.iter().map(|p| &p.name).collect::<Vec<_>>(),
            "body": block_to_json(&func.node.body),
        }),
        Decl::Statement(stmt) => json!({
            "type": "Statement",
            "statement": stmt_to_json(&stmt.node),
        }),
    }
}

fn expr_to_json(expr: &Expr) -> Value {
    match expr {
        Expr::Literal(lit) => json!({
            "type": "Literal",
            "span": JsonSpan::from(lit.span),
            "value": literal_to_json(&lit.node),
        }),
        Expr::Variable(var) => json!({
            "type": "Variable",
            "span": JsonSpan::from(var.span),
            "name": var.node.name,
        }),
        Expr::Unary(unary) => json!({
            "type": "Unary",
            "span": JsonSpan::from(unary.span),
            "op": format!("{:?}", unary.node.op),
            "operand": expr_to_json(&unary.node.operand),
        }),
        Expr::Binary(binary) => json!({
            "type": "Binary",
            "span": JsonSpan::from(binary.span),
            "op": format!("{:?}", binary.node.op),
            "left": expr_to_json(&binary.node.left),
            "right": expr_to_json(&binary.node.right),
        }),
        Expr::Logical(logical) => json!({
            "type": "Logical",
            "span": JsonSpan::from(logical.span),
            "op": format!("{:?}", logical.node.op),
            "left": expr_to_json(&logical.node.left),
            "right": expr_to_json(&logical.node.right),
        }),
        Expr::Assign(assign) => json!({
            "type": "Assign",
            "span": JsonSpan::from(assign.span),
            "name": assign.node.name.name,
            "value": expr_to_json(&assign.node.value),
        }),
        Expr::Call(call) => json!({
            "type": "Call",
            "span": JsonSpan::from(call.span),
            "callee": expr_to_json(&call.node.callee),
            "args": call.node.args.iter().map(expr_to_json).collect::<Vec<_>>(),
        }),
        Expr::Grouping(grouped) => json!({
            "type": "Grouping",
            "span": JsonSpan::from(grouped.span),
            "expression": expr_to_json(&grouped.node),
        }),
    }
}

fn literal_to_json(literal: &Literal) -> Value {
    match literal {
        Literal::Number(n) => json!(n),
        Literal::String(s) => json!(s),
        Literal::Bool(b) => json!(b),
        Literal::Nil => json!(null),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn tree_dump_contains_function_and_return() {
        let program = parse("fn add(a, b) { return a + b; }").unwrap();
        let output = dump_program(&program, DumpFormat::Tree);
        assert!(output.contains("Program"));
        assert!(output.contains("Decl::Function"));
        assert!(output.contains("name: add"));
        assert!(output.contains("Stmt::Return"));
        assert!(output.contains("Binary::Add"));
    }

    #[test]
    fn json_dump_is_valid_and_typed() {
        let program = parse("let x = 1;").unwrap();
        let output = dump_program(&program, DumpFormat::Json);
        let value: Value = serde_json::from_str(&output).expect("valid json");
        assert_eq!(value["type"], "Program");
        assert!(value["declarations"].is_array());
    }
}