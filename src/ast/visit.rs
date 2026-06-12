use super::decl::Decl;
use super::expr::Expr;
use super::stmt::Stmt;
use super::Program;

pub trait Visitor {
    type Output;

    fn visit_program(&mut self, program: &Program) -> Self::Output;
    fn visit_decl(&mut self, decl: &Decl) -> Self::Output;
    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::Output;
    fn visit_expr(&mut self, expr: &Expr) -> Self::Output;
}