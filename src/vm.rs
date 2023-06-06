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
    ($val: ident, $op: tt, $x: ident) => {
        {
            let right = $x.stack.pop().unwrap();
            let left = $x.stack.pop().unwrap();
            if let (Value::NUMBER(x), Value::NUMBER(y)) = (left, right) {
                $x.stack.push((Value::$val(x $op y)));
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
                    binary_op!(NUMBER, +, self);
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
        println!("{:?}", self.peek());
        InterpretResult::InterpretOk
    }

    fn peek(&self) -> Value {
        self.stack.last().unwrap().clone()
    }
    fn read_constant(&self, idx: usize) -> Value {
        self.chunk.constants[idx].clone()
    }
}
