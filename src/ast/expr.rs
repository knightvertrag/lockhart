use super::span::{Ident, Span, Spanned};

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Equal,
    NotEqual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogicalExpr {
    pub op: LogicalOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssignExpr {
    pub name: Ident,
    pub value: Box<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(Spanned<Literal>),
    Variable(Spanned<Ident>),
    Unary(Spanned<UnaryExpr>),
    Binary(Spanned<BinaryExpr>),
    Logical(Spanned<LogicalExpr>),
    Assign(Spanned<AssignExpr>),
    Call(Spanned<CallExpr>),
    Grouping(Spanned<Box<Expr>>),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(s) => s.span,
            Expr::Variable(s) => s.span,
            Expr::Unary(s) => s.span,
            Expr::Binary(s) => s.span,
            Expr::Logical(s) => s.span,
            Expr::Assign(s) => s.span,
            Expr::Call(s) => s.span,
            Expr::Grouping(s) => s.span,
        }
    }
}