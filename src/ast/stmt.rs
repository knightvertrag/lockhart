use super::decl::Decl;
use super::expr::Expr;
use super::span::{Span, Spanned};

#[derive(Clone, Debug, PartialEq)]
pub struct BlockStmt {
    pub declarations: Vec<Spanned<Decl>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForStmt {
    pub initializer: Option<Box<Decl>>,
    pub condition: Option<Expr>,
    pub increment: Option<Expr>,
    pub body: Box<Stmt>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expression(Spanned<Expr>),
    Print(Spanned<Expr>),
    Block(Spanned<BlockStmt>),
    If(Spanned<IfStmt>),
    While(Spanned<WhileStmt>),
    For(Spanned<ForStmt>),
    Return(Spanned<Option<Expr>>),
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Expression(s) => s.span,
            Stmt::Print(s) => s.span,
            Stmt::Block(s) => s.span,
            Stmt::If(s) => s.span,
            Stmt::While(s) => s.span,
            Stmt::For(s) => s.span,
            Stmt::Return(s) => s.span,
        }
    }
}