use std::collections::HashMap;

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
    globals: HashMap<String, Value>,
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

#[derive(Debug)]
pub enum InterpretError {
    InterpretCompileError(&'static str),
    InterpretRuntimeError(&'static str),
}

impl Vm {
    pub fn init_vm() -> Vm {
        let chunk = Chunk::new();
        let ip = 0;
        Vm {
            chunk,
            ip,
            stack: Vec::<Value>::new(),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, source: String) -> Result<(), InterpretError> {
        compile(source, &mut self.chunk).unwrap();
        return self.run();
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            // disassemble_instruction(&self.chunk, _i);

            match self.chunk.code[self.ip].0 {
                Opcode::OP_RETURN => {
                    self.ip += 1;
                    return Ok(());
                }
                Opcode::OP_CONSTANT(idx) => {
                    let constant = self.read_constant(idx);
                    // println!("{:?}", constant);
                    self.stack.push(constant);
                    // return InterpretResult::InterpretOk;
                }
                Opcode::OP_NEGATE => {
                    let to_negate = self.peek(0);
                    if let Value::NUMBER(mut n) = to_negate {
                        n = -n;
                        self.stack.pop();
                        self.stack.push(Value::NUMBER(n));
                        println!("{:?}", self.peek(0));
                    } else {
                        return Err(InterpretError::InterpretRuntimeError(
                            "Failed to negate non-number value",
                        ));
                    }
                }
                Opcode::OP_ADD => {
                    let (x, y) = (self.peek(0), self.peek(1));
                    if let (Value::STR(s1), Value::STR(s2)) = (x, y) {
                        let concatenated = s2.to_owned() + s1;
                        self.stack.pop();
                        self.stack.pop();
                        self.stack.push(Value::STR(concatenated));
                    } else if let (Value::NUMBER(_), Value::NUMBER(_)) = (x, y) {
                        binary_op!(NUMBER, +, self);
                    } else {
                        return Err(InterpretError::InterpretRuntimeError(
                            "Failed to add non-number values",
                        ));
                    }

                    // println!("{:?}", self.peek());
                }
                Opcode::OP_SUBSTRACT => {
                    binary_op!(NUMBER, -, self);
                    // println!("{:?}", self.peek());
                }
                Opcode::OP_DIVIDE => {
                    binary_op!(NUMBER, /, self);
                    // println!("{:?}", self.peek());
                }
                Opcode::OP_MULTIPLY => {
                    binary_op!(NUMBER, *, self);
                    // println!("{:?}", self.peek())
                }
                Opcode::OP_MOD => {
                    binary_op!(NUMBER, %, self);
                    // println!("{:?}", self.peek())
                }
                Opcode::OP_TRUE => self.stack.push(Value::BOOL(true)),
                Opcode::OP_FALSE => self.stack.push(Value::BOOL(false)),
                Opcode::OP_NIL => self.stack.push(Value::NIL),
                Opcode::OP_NOT => {
                    let falsified = Value::BOOL(Value::falsify(&self.stack.pop().unwrap()));
                    self.stack.push(falsified);
                }
                Opcode::OP_EQ => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(Value::BOOL(Value::values_equal(&a, &b)));
                }
                Opcode::OP_GT => {
                    binary_op!(BOOL, >, self);
                }
                Opcode::OP_LT => {
                    binary_op!(BOOL, <, self);
                }
                Opcode::OP_PRINT => {
                    let val = self.stack.pop().unwrap();
                    println!("{}", val);
                }
                Opcode::OP_POP => {
                    self.stack.pop();
                }
                Opcode::OP_DEFINE_GLOBAL(idx) => {
                    let name = self.read_constant(idx).get_string().unwrap().to_owned();
                    let value = self.stack.pop().unwrap();
                    self.globals.insert(name, value);
                }
                Opcode::OP_GET_GLOBAL(idx) => {
                    let name = self.read_constant(idx).get_string().unwrap().to_owned();
                    if let Some(value) = self.globals.get(&name) {
                        self.stack.push(value.clone());
                    } else {
                        return Err(InterpretError::InterpretRuntimeError("Undefined Variable"));
                    }
                }
                Opcode::OP_SET_GLOBAL(idx) => {
                    let name = self.read_constant(idx).get_string().unwrap().to_owned();
                    if self.globals.contains_key(&name) {
                        let value = self.peek(0);
                        self.globals.insert(name, value.clone());
                    } else {
                        return Err(InterpretError::InterpretRuntimeError("Undefined Variable"));
                    }
                }
                Opcode::OP_GET_LOCAL(slot_index) => {
                    self.stack.push(self.stack[slot_index].clone());
                },
                Opcode::OP_SET_LOCAL(slot_index) => {
                    self.stack[slot_index] = self.peek(0).clone();
                },
            }
            self.ip += 1;
        }
        // println!("{:?}", self.peek(0));
    }

    fn peek(&self, idx: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - idx]
    }
    fn read_constant(&self, idx: usize) -> Value {
        self.chunk.constants[idx].clone()
    }
}
