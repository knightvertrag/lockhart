use crate::{vm::Vm, chunk::{Lineno, Chunk}, bytecode::Opcode, value::{Value, self}};
#[cfg(test)]

#[test]
fn negate_chunk() {
    let mut interpreter = Vm::init_vm();
    let mut chunk = Chunk::new();
    let const_ = chunk.add_constant(value::Value::NUMBER(5.0));
    chunk.write_chunk(Opcode::OPCONSTANT(const_), Lineno(0));
    chunk.write_chunk(Opcode::OPNEGATE, Lineno(1));
    chunk.write_chunk(Opcode::OPRETURN, Lineno(2));
    interpreter.interpret(chunk);
    let rhs = Value::NUMBER(-5.0);
    assert_eq!(interpreter.peek(), rhs);
}

#[test]
fn add_chunk() {
    let mut interpreter = Vm::init_vm();
    let mut chunk = Chunk::new();
    let const1 = chunk.add_constant(value::Value::NUMBER(10.0));
    let const2 = chunk.add_constant(value::Value::NUMBER(5.0));
    chunk.write_chunk(Opcode::OPCONSTANT(const1), Lineno(0));
    chunk.write_chunk(Opcode::OPCONSTANT(const2), Lineno(1));
    chunk.write_chunk(Opcode::OPADD, Lineno(2));
    chunk.write_chunk(Opcode::OPRETURN, Lineno(3));
    interpreter.interpret(chunk);
    let rhs = Value::NUMBER(15.0);
    assert_eq!(interpreter.peek(), rhs);
}