use crate::{
    bytecode::Opcode, chunk::{disassemble::disassemble_instruction, Chunk, Lineno}, compiler::compile, gc::{Gc, GcRef}, object::ObjFunction, table::Table, value::Value
};

mod tests;
pub struct Vm {
    gc: Gc,
    frames: [CallFrame; Vm::MAX_FRAMES],
    frame_count: usize,
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: Table,
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

#[derive(Clone, Copy)]
struct CallFrame {
    function: GcRef<ObjFunction>,
    ip: usize,
    slot: usize
}

impl Vm {
    const MAX_FRAMES: usize = 64;

    pub fn init_vm() -> Vm {
        let chunk = Chunk::new();
        let ip = 0;
        let gc = Gc::new();
        Vm {
            gc,
            frames: [
                CallFrame {
                    function: GcRef::dangling(),
                    ip: 0,
                    slot: 0,
                }; Vm::MAX_FRAMES
            ],
            frame_count: 0,
            chunk,
            ip,
            stack: Vec::<Value>::new(),
            globals: Table::new(),
        }
    }

    pub fn interpret(&mut self, source: String) -> Result<(), InterpretError> {
        compile(source, &mut self.gc).unwrap();
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
                        // println!("{:?}", self.peek(0));
                    } else {
                        return Err(InterpretError::InterpretRuntimeError(
                            "Failed to negate non-number value",
                        ));
                    }
                }
                Opcode::OP_ADD => {
                    let (x, y) = (self.peek(0), self.peek(1));
                    if let (Value::STR(s1), Value::STR(s2)) = (x, y) {
                        let concatenated = s2.s.to_owned() + &s1.s;
                        let interned = self.gc.intern(concatenated);
                        self.stack.pop();
                        self.stack.pop();
                        self.stack.push(Value::STR(interned));
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
                    let name = self.read_constant(idx).get_string().unwrap();
                    let value = self.stack.pop().unwrap();
                    self.globals.set(name, value);
                }
                Opcode::OP_GET_GLOBAL(idx) => {
                    let name = self.read_constant(idx).get_string().unwrap();
                    if let Some(value) = self.globals.get(name) {
                        self.stack.push(value.clone());
                    } else {
                        return Err(InterpretError::InterpretRuntimeError("Undefined Variable"));
                    }
                }
                Opcode::OP_SET_GLOBAL(idx) => {
                    let name = self.read_constant(idx).get_string().unwrap();
                    let value = self.peek(0);
                    if self.globals.set(name, value.clone()) {
                        self.globals.delete_entry(name);
                        return Err(InterpretError::InterpretRuntimeError("Undefined Variable"));
                    }
                }
                Opcode::OP_GET_LOCAL(slot_index) => {
                    self.stack.push(self.stack[slot_index].clone());
                }
                Opcode::OP_SET_LOCAL(slot_index) => {
                    self.stack[slot_index] = self.peek(0).clone();
                }
                Opcode::OP_JUMP_IF_FALSE(jump_size) => {
                    if Value::is_falsey(self.peek(0)) {
                        self.ip += jump_size;
                    }
                }
                Opcode::OP_JUMP(jump_size) => {
                    self.ip += jump_size;
                }
                Opcode::OP_LOOP(jump_size) => {
                    self.ip -= jump_size;
                }
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
