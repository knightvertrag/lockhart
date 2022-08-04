use crate::{
    bytecode::Opcode,
    chunk::{Chunk, Constant, Lineno},
    value::Value,
};

mod disassemble;
pub struct Vm {
    chunk: Chunk,
    ip: *const (Opcode, Lineno),
    stack: Vec<Value>,
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
        loop {
            let ip = self.ip;
            unsafe {
                self.ip = self.ip.add(1);
                match (*ip).0 {
                    Opcode::OPRETURN => {
                        self.stack.pop();
                        return InterpretResult::InterpretOk;
                    }
                    Opcode::OPCONSTANT(idx) => {
                        let constant = self.read_constant(idx);
                        println!("{:?}", constant);
                        self.stack.push(constant);
                        // return InterpretResult::InterpretOk;
                    }
                    Opcode::OPNEGATE => {
                        let to_negate = self.peek();
                        match to_negate {
                            Value::NUMBER(mut n) => {
                                n = -n;
                                self.stack.pop();
                                self.stack.push(Value::NUMBER(n));
                                println!("{:?}", self.peek());
                                // return InterpretResult::InterpretOk;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn peek(&self) -> Value {
        return self.stack[self.stack.len() - 1].clone();
    }

    fn read_constant(&self, idx: usize) -> Value {
        let constant = self.chunk.constants[idx].clone();
        match constant {
            Constant::DOUBLE(n) => return Value::NUMBER(n),
            Constant::STRING(s) => return Value::STRING(s),
        }
    }
}
