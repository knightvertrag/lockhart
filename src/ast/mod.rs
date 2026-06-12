pub mod decl;
pub mod expr;
pub mod span;
pub mod stmt;
pub mod visit;

pub use decl::{Decl, FnDecl, VarDecl};
pub use expr::{
    AssignExpr, BinaryExpr, BinaryOp, CallExpr, Expr, Literal, LogicalExpr, LogicalOp, UnaryExpr,
    UnaryOp,
};
pub use span::{Ident, Span, Spanned};
pub use stmt::{BlockStmt, ForStmt, IfStmt, Stmt, WhileStmt};

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub declarations: Vec<Spanned<Decl>>,
}