use crate::{
    lexer::Lexer,
    token::{Delimiters, Ident, Keywords, Operators, Token, TokenType},
};

use self::ast::{LetStatement, Statement};

mod ast;

pub struct Parser {
    lexer: Box<Lexer>,
    cur_token: Token,
    peek_token: Token,
}

impl Parser {
    fn new(lexer: Lexer) -> Box<Parser> {
        let mut p = Parser {
            lexer: Box::new(lexer),
            cur_token: Token::new_def(),
            peek_token: Token::new_def(),
        };

        p.next_token();
        p.next_token();

        Box::new(p)
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn parse_statement(&mut self) -> Option<ast::Node> {
        let statement: ast::Node;
        match self.cur_token.type_ {
            TokenType::KEYWORDS(Keywords::LET) => statement = self.parse_let_statement().unwrap(),
            _ => return None,
        }
        Some(statement)
    }

    fn parse_let_statement(&mut self) -> Option<ast::Node> {
        if self.peek_token.type_ != TokenType::IDENT(Ident::IDENT) {
            return None;
        }
        self.next_token();
        let name = Box::new(ast::Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        });
        if self.peek_token.type_ != TokenType::OPERATORS(Operators::ASSIGN) {
            return None;
        }
        loop {
            // @TODO: need to parse expression here
            if self.cur_token.type_ == TokenType::DELIMITERS(Delimiters::SEMICOLON) {
                break;
            }
            self.next_token();
        }
        let stmnt = ast::LetStatement {
            token: self.cur_token.clone(),
            name,
        };

        Some(ast::Node::Statement(Statement::LET(stmnt)))
    }

    fn parse_program(&mut self) -> Box<ast::Program> {
        let mut statements: Vec<Box<ast::Node>> = Vec::new();
        while self.cur_token.type_ != TokenType::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(Box::new(stmt));
            }
            self.next_token();
        }
        let prog = ast::Program { statements };
        Box::new(prog)
    }
}

impl Iterator for Parser {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_token.type_ == TokenType::EOF {
            return None;
        }
        self.next_token();
        Some(self.cur_token.clone())
    }
}
