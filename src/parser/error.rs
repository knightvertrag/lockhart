use std::fmt;

use crate::ast::Span;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedToken {
        expected: &'static str,
        found: Token,
    },
    UnexpectedEnd {
        expected: &'static str,
        span: Span,
    },
    InvalidAssignment {
        span: Span,
    },
    DuplicateLocal {
        name: String,
        span: Span,
    },
    ReturnOutsideFunction {
        span: Span,
    },
    TooManyLocals {
        span: Span,
    },
    TooManyParameters {
        span: Span,
    },
    TooManyArguments {
        span: Span,
    },
    InvalidNumber {
        span: Span,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found } => {
                write!(
                    f,
                    "[line {}] Expected {}, found '{}'",
                    found.lineno, expected, found.literal
                )
            }
            ParseError::UnexpectedEnd { expected, span } => {
                write!(
                    f,
                    "[line {}] Expected {} at end of input",
                    span.line, expected
                )
            }
            ParseError::InvalidAssignment { span } => {
                write!(f, "[line {}] Invalid assignment target", span.line)
            }
            ParseError::DuplicateLocal { name, span } => {
                write!(
                    f,
                    "[line {}] Variable '{}' already declared in this scope",
                    span.line, name
                )
            }
            ParseError::ReturnOutsideFunction { span } => {
                write!(f, "[line {}] Cannot return from top-level code", span.line)
            }
            ParseError::TooManyLocals { span } => {
                write!(f, "[line {}] Too many local variables", span.line)
            }
            ParseError::TooManyParameters { span } => {
                write!(f, "[line {}] Too many parameters", span.line)
            }
            ParseError::TooManyArguments { span } => {
                write!(f, "[line {}] Too many arguments", span.line)
            }
            ParseError::InvalidNumber { span } => {
                write!(f, "[line {}] Invalid number literal", span.line)
            }
        }
    }
}

impl std::error::Error for ParseError {}