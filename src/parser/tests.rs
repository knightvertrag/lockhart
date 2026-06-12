use crate::ast::{BinaryOp, Decl, Expr, Literal, Stmt};
use crate::parser::{parse, ParseError};

#[test]
fn parses_number_literal() {
    let program = parse("1;").unwrap();
    assert_eq!(program.declarations.len(), 1);
    match &program.declarations[0].node {
        Decl::Statement(stmt) => match &stmt.node {
            Stmt::Expression(expr) => match &expr.node {
                Expr::Literal(lit) => assert_eq!(lit.node, Literal::Number(1.0)),
                _ => panic!("expected literal"),
            },
            _ => panic!("expected expression stmt"),
        },
        _ => panic!("expected statement decl"),
    }
}

#[test]
fn parses_precedence() {
    let program = parse("1 + 2 * 3;").unwrap();
    match &program.declarations[0].node {
        Decl::Statement(stmt) => match &stmt.node {
            Stmt::Expression(expr) => match &expr.node {
            Expr::Binary(binary) => {
                assert_eq!(binary.node.op, BinaryOp::Add);
                match &*binary.node.left {
                    Expr::Literal(left) => assert_eq!(left.node, Literal::Number(1.0)),
                    _ => panic!("expected left literal"),
                }
                match &*binary.node.right {
                    Expr::Binary(right) => {
                        assert_eq!(right.node.op, BinaryOp::Multiply);
                    }
                    _ => panic!("expected right binary"),
                }
            }
            _ => panic!("expected binary expr"),
            },
            _ => panic!("expected expression stmt"),
        },
        _ => panic!("expected stmt decl"),
    }
}

#[test]
fn parses_function_declaration() {
    let program = parse("fn add(a, b) { return a + b; }").unwrap();
    match &program.declarations[0].node {
        Decl::Function(func) => {
            assert_eq!(func.node.name.name, "add");
            assert_eq!(func.node.params.len(), 2);
        }
        _ => panic!("expected function decl"),
    }
}

#[test]
fn parses_for_loop() {
    let program = parse("for (let i = 0; i < 5; i = i + 1) { print i; }").unwrap();
    match &program.declarations[0].node {
        Decl::Statement(stmt) => match &stmt.node {
            Stmt::For(for_stmt) => {
                assert!(for_stmt.node.initializer.is_some());
                assert!(for_stmt.node.condition.is_some());
                assert!(for_stmt.node.increment.is_some());
            }
            _ => panic!("expected for stmt"),
        },
        _ => panic!("expected stmt decl"),
    }
}

#[test]
fn return_outside_function_is_parse_error() {
    let err = parse("return 1;").unwrap_err();
    assert!(matches!(err, ParseError::ReturnOutsideFunction { .. }));
}