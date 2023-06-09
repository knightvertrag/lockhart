use std::thread::panicking;

use crate::{
    bytecode::Opcode,
    chunk::{disassemble::disassemble_instruction, Chunk, Lineno},
    compiler::compile,
    value::Value,
};

mod tests;
pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

macro_rules! binary_op {
    ($ret: ident, $op: tt, $x: ident) => {
        {
            let right = $x.stack.pop().unwrap();
            let left = $x.stack.pop().unwrap();
            if let (Value::NUMBER(x), Value::NUMBER(y)) = (left, right) {
                $x.stack.push((Value::$ret(x $op y)));
            } else {
                panic!("Mismatched Types");
            }
        }
    }
}

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

impl Vm {
    pub fn init_vm() -> Vm {
        let chunk = Chunk::new();
        let ip = 0;
        Vm {
            chunk,
            ip,
            stack: Vec::<Value>::new(),
        }
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        compile(source, &mut self.chunk).unwrap();
        return self.run();
    }

    fn run(&mut self) -> InterpretResult {
        for _i in 0..self.chunk.code.len() {
            disassemble_instruction(&self.chunk, _i);

            match self.chunk.code[self.ip].0 {
                Opcode::OPRETURN => {
                    // self.stack.pop();
                    break;
                }
                Opcode::OPCONSTANT(idx) => {
                    let constant = self.read_constant(idx);
                    // println!("{:?}", constant);
                    self.stack.push(constant);
                    // return InterpretResult::InterpretOk;
                }
                Opcode::OPNEGATE => {
                    let to_negate = self.peek(0);
                    if let Value::NUMBER(mut n) = to_negate {
                        n = -n;
                        self.stack.pop();
                        self.stack.push(Value::NUMBER(n));
                        println!("{:?}", self.peek(0));
                    } else {
                        panic!("Cannot negate a non-number value");
                    }
                }
                Opcode::OPADD => {
                    let (x, y) = (self.peek(0), self.peek(1));
                    if let (Value::STR(s1), Value::STR(s2)) = (x, y) {
                        let concatenated = s2.to_owned() + s1;
                        self.stack.pop();
                        self.stack.pop();
                        self.stack.push(Value::STR(concatenated));
                    } else if let (Value::NUMBER(_), Value::NUMBER(_)) = (x, y) {
                        binary_op!(NUMBER, +, self);
                    } else {
                        panic!("Failure to add, operation must be between Strings or Numbers");
                    }

                    // println!("{:?}", self.peek());
                }
                Opcode::OPSUBSTRACT => {
                    binary_op!(NUMBER, -, self);
                    // println!("{:?}", self.peek());
                }
                Opcode::OPDIVIDE => {
                    binary_op!(NUMBER, /, self);
                    // println!("{:?}", self.peek());
                }
                Opcode::OPMULTIPLY => {
                    binary_op!(NUMBER, *, self);
                    // println!("{:?}", self.peek())
                }
                Opcode::OPMOD => {
                    binary_op!(NUMBER, %, self);
                    // println!("{:?}", self.peek())
                }
                Opcode::OPTRUE => self.stack.push(Value::BOOL(true)),
                Opcode::OPFALSE => self.stack.push(Value::BOOL(false)),
                Opcode::OPNIL => self.stack.push(Value::NIL),
                Opcode::OPNOT => {
                    let falsified = Value::BOOL(Value::falsify(&self.stack.pop().unwrap()));
                    self.stack.push(falsified);
                }
                Opcode::OPEQ => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(Value::BOOL(Value::values_equal(&a, &b)));
                }
                Opcode::OPGT => {
                    binary_op!(BOOL, >, self);
                }
                Opcode::OPLT => {
                    binary_op!(BOOL, <, self);
                }
            }
            self.ip += 1;
        }
        println!("{:?}", self.peek(0));
        InterpretResult::InterpretOk
    }

    fn peek(&self, idx: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - idx]
    }
    fn read_constant(&self, idx: usize) -> Value {
        self.chunk.constants[idx].clone()
    }
}
