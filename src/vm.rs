use std::thread::panicking;

use crate::{
    bytecode::Opcode,
    chunk::{Chunk, Lineno},
    value::Value,
};

mod disassemble;
mod tests;
pub struct Vm {
    chunk: Chunk,
    ip: *const (Opcode, Lineno),
    stack: Vec<Value>,
}

macro_rules! binary_op {
    ($op: tt, $x: ident) => {
        {
            let right = $x.stack.pop().unwrap();
            let left = $x.stack.pop().unwrap();
            if let (Value::NUMBER(x), Value::NUMBER(y)) = (left, right) {
                $x.stack.push((Value::NUMBER(x $op y)));
            } else {
                panic!("Expected type number");
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
        let ip = chunk.code.as_ptr();
        Vm {
            chunk,
            ip,
            stack: Vec::<Value>::new(),
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.chunk = chunk;
        self.ip = self.chunk.code.as_ptr();
        return self.run();
    }

    fn run(&mut self) -> InterpretResult {
        for _i in 0..self.chunk.code.len() {
            let ip = self.ip;
            unsafe {
                self.ip = self.ip.add(1);
                match (*ip).0 {
                    Opcode::OPRETURN => {
                        // self.stack.pop();
                        break;
                    }
                    Opcode::OPCONSTANT(idx) => {
                        let constant = self.read_constant(idx);
                        println!("{:?}", constant);
                        self.stack.push(constant);
                        // return InterpretResult::InterpretOk;
                    }
                    Opcode::OPNEGATE => {
                        let to_negate = self.peek();
                        if let Value::NUMBER(mut n) = to_negate {
                            n = -n;
                            self.stack.pop();
                            self.stack.push(Value::NUMBER(n));
                            println!("{:?}", self.peek());
                        } else {
                            panic!("Cannot negate a non-number value");
                        }
                    }
                    Opcode::OPADD => {
                        binary_op!(+, self);
                        println!("{:?}", self.peek());
                    }
                    Opcode::OPSUBSTRACT => {
                        binary_op!(-, self);
                        println!("{:?}", self.peek());
                    }
                    Opcode::OPDIVIDE => {
                        binary_op!(/, self);
                        println!("{:?}", self.peek());
                    }
                    Opcode::OPMULTIPLY => {
                        binary_op!(*, self);
                        println!("{:?}", self.peek())
                    }
                    Opcode::OPMOD => {
                        binary_op!(%, self);
                        println!("{:?}", self.peek())
                    }
                }
            }
        }
        InterpretResult::InterpretOk
    }

    fn peek(&self) -> Value {
        return self.stack.last().unwrap().clone();
    }
    fn read_constant(&self, idx: usize) -> Value {
        let constant = self.chunk.constants[idx].clone();
        match constant {
            Value::NUMBER(n) => return Value::NUMBER(n),
            Value::STRING(s) => return Value::STRING(s),
        }
    }
}
