use crate::value::Value;

use super::{InterpretError, Vm};

fn run(source: &str) -> Vm {
    let mut vm = Vm::init_vm();
    let result = vm.interpret(source.to_string());
    assert!(result.is_ok(), "expected program to run, got: {:?}", result);
    vm
}

fn run_err(source: &str) -> InterpretError {
    let mut vm = Vm::init_vm();
    vm.interpret(source.to_string())
        .expect_err("expected runtime error")
}

fn global(vm: &mut Vm, name: &str) -> Value {
    let key = vm.gc.intern(name.to_string());
    vm.globals
        .get(key)
        .unwrap_or_else(|| panic!("missing global '{name}'"))
}

#[test]
fn arithmetic_and_precedence() {
    let mut vm = run("let result = 1 + 2 * 3 - 4 / 2; ");
    assert_eq!(global(&mut vm, "result").get_number(), Some(5.0));
}

#[test]
fn string_concat() {
    let mut vm = run("let msg = \"hello\" + \" world\";");
    let value = global(&mut vm, "msg");
    assert_eq!(value.get_string().map(|s| s.s.clone()), Some("hello world".to_string()));
}

#[test]
fn boolean_logic_and_not() {
    let mut vm = run("let a = true and false; let b = false or true; let c = !false;");
    assert_eq!(global(&mut vm, "a").get_bool(), Some(false));
    assert_eq!(global(&mut vm, "b").get_bool(), Some(true));
    assert_eq!(global(&mut vm, "c").get_bool(), Some(true));
}

#[test]
fn if_else_branching() {
    let mut vm = run("let x = 0; if (true) x = 2; else x = 1;");
    assert_eq!(global(&mut vm, "x").get_number(), Some(2.0));
}

#[test]
fn while_loop_executes() {
    let mut vm = run("let i = 0; let sum = 0; while (i < 4) { sum = sum + i; i = i + 1; }");
    assert_eq!(global(&mut vm, "i").get_number(), Some(4.0));
    assert_eq!(global(&mut vm, "sum").get_number(), Some(6.0));
}

#[test]
fn for_loop_executes() {
    let mut vm = run("let sum = 0; for (let i = 0; i < 5; i = i + 1) { sum = sum + i; }");
    assert_eq!(global(&mut vm, "sum").get_number(), Some(10.0));
}

#[test]
fn function_call_and_return_value() {
    let mut vm = run("fn add(a, b) { return a + b; } let out = add(2, 3);");
    assert_eq!(global(&mut vm, "out").get_number(), Some(5.0));
}

#[test]
fn block_assignment_updates_global() {
    let mut vm = run("let x = 1; { x = 2; } let y = x;");
    assert_eq!(global(&mut vm, "y").get_number(), Some(2.0));
}

#[test]
fn undefined_variable_returns_runtime_error() {
    let err = run_err("x = 1;");
    match err {
        InterpretError::InterpretRuntimeError(msg) => assert_eq!(msg, "Undefined Variable"),
        _ => panic!("expected runtime error"),
    }
}

#[test]
fn wrong_arity_returns_runtime_error() {
    let err = run_err("fn id(a) { return a; } let x = id();");
    match err {
        InterpretError::InterpretRuntimeError(msg) => {
            assert_eq!(msg, "Expected 1 args but found 0")
        }
        _ => panic!("expected runtime error"),
    }
}

#[test]
fn adding_invalid_operands_returns_runtime_error() {
    let err = run_err("let x = true + 1;");
    match err {
        InterpretError::InterpretRuntimeError(msg) => {
            assert_eq!(msg, "Invalid addition operands")
        }
        _ => panic!("expected runtime error"),
    }
}
