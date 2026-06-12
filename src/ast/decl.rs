use super::expr::Expr;
use super::span::{Ident, Span, Spanned};
use super::stmt::{BlockStmt, Stmt};

#[derive(Clone, Debug, PartialEq)]
pub struct VarDecl {
    pub name: Ident,
    pub initializer: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FnDecl {
    pub name: Ident,
    pub params: Vec<Ident>,
    pub body: BlockStmt,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    Var(Spanned<VarDecl>),
    Function(Spanned<FnDecl>),
    Statement(Spanned<Stmt>),
}

impl Decl {
    pub fn span(&self) -> Span {
        match self {
            Decl::Var(s) => s.span,
            Decl::Function(s) => s.span,
            Decl::Statement(s) => s.span,
        }
    }
}