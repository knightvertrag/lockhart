mod error;
mod parse_rule;
mod precedence;
#[cfg(test)]
mod tests;

pub use error::ParseError;

use std::mem::transmute;

use crate::ast::{
    AssignExpr, BinaryExpr, BinaryOp, BlockStmt, CallExpr, Decl, Expr, FnDecl, ForStmt, Ident,
    IfStmt, Literal, LogicalExpr, LogicalOp, Program, Span, Spanned, Stmt, UnaryExpr, UnaryOp,
    VarDecl, WhileStmt,
};
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};

use parse_rule::{ParseRule, RULES};
use precedence::Precedence;

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let mut parser = Parser::new(Lexer::new(source.to_string()));
    parser.advance();
    parser.parse_program()
}

pub struct Parser {
    previous: Token,
    current: Token,
    lexer: Lexer,
    in_function: bool,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Parser {
            previous: Token::new_def(),
            current: Token::new_def(),
            lexer,
            in_function: false,
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        self.current = self.lexer.next_token();
    }

    fn check(&self, type_: TokenType) -> bool {
        self.current.type_ == type_
    }

    fn match_token(&mut self, type_: TokenType) -> bool {
        if self.check(type_) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, type_: TokenType, expected: &'static str) -> Result<(), ParseError> {
        if self.check(type_) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected,
                found: self.current.clone(),
            })
        }
    }

    fn span_since(&self, start: usize, line: usize) -> Span {
        Span::new(start, self.previous.end, line)
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut declarations = Vec::new();
        while !self.check(TokenType::EOF) {
            let start = self.current.start;
            let line = self.current.lineno;
            let decl = self.parse_declaration()?;
            declarations.push(Spanned::new(decl, self.span_since(start, line)));
        }
        Ok(Program { declarations })
    }

    fn parse_declaration(&mut self) -> Result<Decl, ParseError> {
        if self.match_token(TokenType::FUNCTION) {
            self.parse_function_declaration()
        } else if self.match_token(TokenType::LET) {
            self.parse_var_declaration()
        } else {
            let stmt = self.parse_statement()?;
            let span = stmt.span();
            Ok(Decl::Statement(Spanned::new(stmt, span)))
        }
    }

    fn parse_var_declaration(&mut self) -> Result<Decl, ParseError> {
        let start = self.previous.start;
        let line = self.previous.lineno;
        let name = self.parse_ident("Expected variable name")?;
        let initializer = if self.match_token(TokenType::ASSIGN) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume(TokenType::SEMICOLON, "';' after variable declaration")?;
        Ok(Decl::Var(Spanned::new(
            VarDecl {
                name,
                initializer,
            },
            self.span_since(start, line),
        )))
    }

    fn parse_function_declaration(&mut self) -> Result<Decl, ParseError> {
        let start = self.previous.start;
        let line = self.previous.lineno;
        let name = self.parse_ident("Expected function name")?;
        let saved = self.in_function;
        self.in_function = true;
        let params = self.parse_parameters()?;
        self.consume(TokenType::LBRACE, "'{' before function body")?;
        let body = self.parse_block()?;
        self.in_function = saved;
        Ok(Decl::Function(Spanned::new(
            FnDecl { name, params, body },
            self.span_since(start, line),
        )))
    }

    fn parse_parameters(&mut self) -> Result<Vec<Ident>, ParseError> {
        self.consume(TokenType::LPAREN, "'(' after function name")?;
        let mut params = Vec::new();
        if !self.check(TokenType::RPAREN) {
            loop {
                if params.len() >= u8::MAX as usize {
                    return Err(ParseError::TooManyParameters {
                        span: self.current.span(),
                    });
                }
                params.push(self.parse_ident("Expected parameter name")?);
                if !self.match_token(TokenType::COMMA) {
                    break;
                }
            }
        }
        self.consume(TokenType::RPAREN, "')' after parameters")?;
        Ok(params)
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(TokenType::PRINT) {
            self.parse_print_statement()
        } else if self.match_token(TokenType::LBRACE) {
            let start = self.previous.start;
            let line = self.previous.lineno;
            let block = self.parse_block()?;
            Ok(Stmt::Block(Spanned::new(block, self.span_since(start, line))))
        } else if self.match_token(TokenType::IF) {
            self.parse_if_statement()
        } else if self.match_token(TokenType::RETURN) {
            self.parse_return_statement()
        } else if self.match_token(TokenType::WHILE) {
            self.parse_while_statement()
        } else if self.match_token(TokenType::FOR) {
            self.parse_for_statement()
        } else {
            self.parse_expression_statement()
        }
    }

    fn parse_print_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.previous.start;
        let line = self.previous.lineno;
        let expr = self.parse_expression()?;
        self.consume(TokenType::SEMICOLON, "';' after print value")?;
        Ok(Stmt::Print(Spanned::new(expr, self.span_since(start, line))))
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.current.start;
        let line = self.current.lineno;
        let expr = self.parse_expression()?;
        self.consume(TokenType::SEMICOLON, "';' after expression")?;
        Ok(Stmt::Expression(Spanned::new(expr, self.span_since(start, line))))
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.previous.start;
        let line = self.previous.lineno;
        self.consume(TokenType::LPAREN, "'(' before condition")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RPAREN, "')' after condition")?;
        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = if self.match_token(TokenType::ELSE) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(Stmt::If(Spanned::new(
            IfStmt {
                condition,
                then_branch,
                else_branch,
            },
            self.span_since(start, line),
        )))
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.previous.start;
        let line = self.previous.lineno;
        self.consume(TokenType::LPAREN, "'(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RPAREN, "')' after while condition")?;
        let body = Box::new(self.parse_statement()?);
        Ok(Stmt::While(Spanned::new(
            WhileStmt { condition, body },
            self.span_since(start, line),
        )))
    }

    fn parse_for_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.previous.start;
        let line = self.previous.lineno;
        self.consume(TokenType::LPAREN, "'(' after 'for'")?;

        let initializer = if self.match_token(TokenType::SEMICOLON) {
            None
        } else if self.match_token(TokenType::LET) {
            Some(Box::new(self.parse_var_declaration()?))
        } else {
            let stmt = self.parse_expression_statement()?;
            let span = stmt.span();
            Some(Box::new(Decl::Statement(Spanned::new(stmt, span))))
        };

        let condition = if self.check(TokenType::SEMICOLON) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(TokenType::SEMICOLON, "';' after for condition")?;

        let increment = if self.check(TokenType::RPAREN) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(TokenType::RPAREN, "')' after for clauses")?;

        let body = Box::new(self.parse_statement()?);
        Ok(Stmt::For(Spanned::new(
            ForStmt {
                initializer,
                condition,
                increment,
                body,
            },
            self.span_since(start, line),
        )))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        let start = self.previous.start;
        let line = self.previous.lineno;
        if !self.in_function {
            return Err(ParseError::ReturnOutsideFunction {
                span: self.previous.span(),
            });
        }
        let value = if self.check(TokenType::SEMICOLON) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(TokenType::SEMICOLON, "';' after return value")?;
        Ok(Stmt::Return(Spanned::new(value, self.span_since(start, line))))
    }

    fn parse_block(&mut self) -> Result<BlockStmt, ParseError> {
        let mut declarations = Vec::new();
        while !self.check(TokenType::RBRACE) && !self.check(TokenType::EOF) {
            let start = self.current.start;
            let line = self.current.lineno;
            let decl = self.parse_declaration()?;
            declarations.push(Spanned::new(decl, self.span_since(start, line)));
        }
        self.consume(TokenType::RBRACE, "'}' after block")?;
        Ok(BlockStmt { declarations })
    }

    fn parse_ident(&mut self, expected: &'static str) -> Result<Ident, ParseError> {
        if self.check(TokenType::IDENT) {
            let token = self.current.clone();
            self.advance();
            Ok(Ident::new(token.literal.clone(), token.span()))
        } else {
            Err(ParseError::UnexpectedToken {
                expected,
                found: self.current.clone(),
            })
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_precedence(Precedence::PrecAssignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expr, ParseError> {
        self.advance();
        let prefix_rule = RULES[self.previous.type_ as usize];
        let can_assign = precedence <= Precedence::PrecAssignment;
        let mut left = if let Some(prefix) = prefix_rule.prefix {
            prefix(self, can_assign)?
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "expression",
                found: self.previous.clone(),
            });
        };

        while precedence <= ParseRule::get_rule(self.current.type_).precedence {
            let infix_rule = ParseRule::get_rule(self.current.type_);
            self.advance();
            if let Some(infix) = infix_rule.infix {
                left = infix(self, can_assign, left)?;
            }
        }

        if can_assign && self.match_token(TokenType::ASSIGN) {
            return Err(ParseError::InvalidAssignment { span: left.span() });
        }

        Ok(left)
    }

    fn parse_number_expr(&mut self, _: bool) -> Result<Expr, ParseError> {
        let span = self.previous.span();
        let value = self
            .previous
            .literal
            .parse::<f64>()
            .map_err(|_| ParseError::InvalidNumber { span })?;
        Ok(Expr::Literal(Spanned::new(Literal::Number(value), span)))
    }

    fn parse_string_expr(&mut self, _: bool) -> Result<Expr, ParseError> {
        let span = self.previous.span();
        Ok(Expr::Literal(Spanned::new(
            Literal::String(self.previous.literal.clone()),
            span,
        )))
    }

    fn parse_literal_expr(&mut self, _: bool) -> Result<Expr, ParseError> {
        let span = self.previous.span();
        let literal = match self.previous.type_ {
            TokenType::TRUE => Literal::Bool(true),
            TokenType::FALSE => Literal::Bool(false),
            TokenType::NIL => Literal::Nil,
            _ => unreachable!(),
        };
        Ok(Expr::Literal(Spanned::new(literal, span)))
    }

    fn parse_grouping_expr(&mut self, _: bool) -> Result<Expr, ParseError> {
        let open = self.previous.span();
        let expr = self.parse_expression()?;
        self.consume(TokenType::RPAREN, "')' after expression")?;
        let span = open.merge(self.previous.span());
        Ok(Expr::Grouping(Spanned::new(Box::new(expr), span)))
    }

    fn parse_unary_expr(&mut self, _: bool) -> Result<Expr, ParseError> {
        let op_token = self.previous.clone();
        let operand = self.parse_precedence(Precedence::PrecUnary)?;
        let span = op_token.span().merge(operand.span());
        let op = match op_token.type_ {
            TokenType::MINUS => UnaryOp::Negate,
            TokenType::NOT => UnaryOp::Not,
            _ => unreachable!(),
        };
        Ok(Expr::Unary(Spanned::new(
            UnaryExpr {
                op,
                operand: Box::new(operand),
            },
            span,
        )))
    }

    fn parse_binary_expr(&mut self, _: bool, left: Expr) -> Result<Expr, ParseError> {
        let op_token = self.previous.clone();
        let rule = ParseRule::get_rule(op_token.type_);
        let next_precedence = if (rule.precedence as usize) < 11 {
            unsafe { transmute(rule.precedence as i8 + 1) }
        } else {
            rule.precedence
        };
        let right = self.parse_precedence(next_precedence)?;
        let span = left.span().merge(op_token.span()).merge(right.span());
        let op = match op_token.type_ {
            TokenType::PLUS => BinaryOp::Add,
            TokenType::MINUS => BinaryOp::Subtract,
            TokenType::MUL => BinaryOp::Multiply,
            TokenType::DIV => BinaryOp::Divide,
            TokenType::GT => BinaryOp::Greater,
            TokenType::LT => BinaryOp::Less,
            TokenType::GEQ => BinaryOp::GreaterEqual,
            TokenType::LEQ => BinaryOp::LessEqual,
            TokenType::EQ => BinaryOp::Equal,
            TokenType::NEQ => BinaryOp::NotEqual,
            _ => unreachable!(),
        };
        Ok(Expr::Binary(Spanned::new(
            BinaryExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        )))
    }

    fn parse_logical_expr(&mut self, _: bool, left: Expr) -> Result<Expr, ParseError> {
        let op_token = self.previous.clone();
        let op = match op_token.type_ {
            TokenType::AND => LogicalOp::And,
            TokenType::OR => LogicalOp::Or,
            _ => unreachable!(),
        };
        let next = match op {
            LogicalOp::And => Precedence::PrecAnd,
            LogicalOp::Or => Precedence::PrecOr,
        };
        let right = self.parse_precedence(next)?;
        let span = left.span().merge(op_token.span()).merge(right.span());
        Ok(Expr::Logical(Spanned::new(
            LogicalExpr {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            span,
        )))
    }

    fn parse_variable_expr(&mut self, can_assign: bool) -> Result<Expr, ParseError> {
        let name = Ident::new(self.previous.literal.clone(), self.previous.span());
        if can_assign && self.match_token(TokenType::ASSIGN) {
            let value = self.parse_expression()?;
            let span = name.span.merge(value.span());
            return Ok(Expr::Assign(Spanned::new(
                AssignExpr {
                    name,
                    value: Box::new(value),
                },
                span,
            )));
        }
        let span = name.span;
        Ok(Expr::Variable(Spanned::new(name, span)))
    }

    fn parse_call_expr(&mut self, _: bool, callee: Expr) -> Result<Expr, ParseError> {
        let callee_span = callee.span();
        let mut args = Vec::new();
        if !self.check(TokenType::RPAREN) {
            loop {
                if args.len() >= u8::MAX as usize {
                    return Err(ParseError::TooManyArguments {
                        span: self.current.span(),
                    });
                }
                args.push(self.parse_expression()?);
                if !self.match_token(TokenType::COMMA) {
                    break;
                }
            }
        }
        self.consume(TokenType::RPAREN, "')' after arguments")?;
        let span = callee_span.merge(self.previous.span());
        Ok(Expr::Call(Spanned::new(
            CallExpr {
                callee: Box::new(callee),
                args,
            },
            span,
        )))
    }
}