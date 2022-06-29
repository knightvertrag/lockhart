use crate::token::{self, Token, TokenType};

pub enum Node {
    Statement(Statement),
    Expression,
}

pub enum Statement {
    LET(LetStatement),
    RETURN
}
pub struct Program {
    pub statements: Vec<Box<Node>>,
}


pub struct LetStatement {
    pub token: Token,
    pub name: Box<Identifier>,
    // value: Box<Node>,
}

pub struct Identifier {
    pub token: Token,
    pub value: String,
}