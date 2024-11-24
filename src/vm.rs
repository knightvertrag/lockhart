use std::{cell::RefCell, cmp::max, ptr::null};

use crate::{
    bytecode::Opcode,
    chunk::{disassemble::disassemble_instruction, Chunk, Lineno},
    compiler::compile,
    gc::{Gc, GcRef},
    object::ObjFunction,
    table::Table,
    value::Value,
};

mod tests;
pub struct Vm {
    gc: Gc,
    frames: [CallFrame; Vm::MAX_FRAMES],
    frame_count: usize,
    stack: Vec<Value>,
    stack_top: usize,
    globals: Table,
}

macro_rules! binary_op {
    ($ret: ident, $op: tt, $x: ident) => {
        {
            let right = $x.pop();
            let left = $x.pop();
            if let (Value::NUMBER(x), Value::NUMBER(y)) = (left, right) {
                $x.push((Value::$ret(x $op y)));
            } else {
                panic!("Mismatched Types");
            }
        }
    }
}

#[derive(Debug)]
pub enum InterpretError {
    InterpretCompileError(String),
    InterpretRuntimeError(String),
}

#[derive(Clone, Copy)]
struct CallFrame {
    function: GcRef<ObjFunction>,
    ip: *const (Opcode, Lineno), // pointer to instruction vector
    slot: usize,                 // starting stack-slot index of this function call
}

impl CallFrame {
    pub fn new(function: GcRef<ObjFunction>, slot: usize) -> CallFrame {
        CallFrame {
            function,
            ip: function.chunk.code.as_ptr(),
            slot,
        }
    }

    pub fn offset(&self) -> usize {
        unsafe {
            let pos = self.ip.offset_from(self.function.chunk.code.as_ptr());
            pos as usize
        }
    }
}

impl Vm {
    const MAX_FRAMES: usize = 64;
    const MAX_STACK: usize = 255;
    pub fn init_vm() -> Vm {
        let gc = Gc::new();
        Vm {
            gc,
            frames: [CallFrame {
                function: GcRef::dangling(),
                ip: null(),
                slot: 0,
            }; Vm::MAX_FRAMES],
            frame_count: 0,
            stack: vec![Value::NIL; Vm::MAX_STACK],
            stack_top: 0,
            globals: Table::new(),
        }
    }

    pub fn interpret(&mut self, source: String) -> Result<(), InterpretError> {
        let function = compile(source, &mut self.gc)?;
        self.push(Value::FUNCTION(function));
        // let closure = self.alloc(function);
        // let frame = CallFrame::new(*closure, 0);
        // self.frames[self.frame_count] = frame;
        // self.frame_count += 1;
        self.call(function, 0)?;
        return self.run();
    }

    fn alloc<T>(&mut self, object: T) -> GcRef<T> {
        self.gc.alloc(object)
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        unsafe {
            let mut frame = &mut *(&mut self.frames[self.frame_count - 1] as *mut CallFrame);
            loop {
                // disassemble_instruction(&frame.function.chunk, _i);
                let op = (*frame.ip).0;
                // println!("{:?}", op);
                frame.ip = frame.ip.offset(1);
                match op {
                    Opcode::OP_RETURN => {
                        let returned_value = self.pop();
                        self.frame_count -= 1;
                        if self.frame_count == 0 {
                            self.pop();
                            return Ok(());
                        }
                        self.stack_top = frame.slot;
                        self.push(returned_value);
                        frame = &mut *(&mut self.frames[self.frame_count - 1] as *mut CallFrame);
                    }
                    Opcode::OP_CONSTANT(idx) => {
                        let constant = Vm::read_constant(&frame, idx);
                        // println!("{}", constant);
                        self.push(constant);
                        // return InterpretResult::InterpretOk;
                    }
                    Opcode::OP_NEGATE => {
                        let to_negate = self.peek(0);
                        if let Value::NUMBER(mut n) = to_negate {
                            n = -n;
                            self.pop();
                            self.push(Value::NUMBER(n));
                            // println!("{:?}", self.peek(0));
                        } else {
                            return Err(InterpretError::InterpretRuntimeError(
                                "Failed to negate non-number value".to_string(),
                            ));
                        }
                    }
                    Opcode::OP_ADD => {
                        let (x, y) = (self.peek(0), self.peek(1));
                        if let (Value::STR(s1), Value::STR(s2)) = (x, y) {
                            let concatenated = s2.s.to_owned() + &s1.s;
                            let interned = self.gc.intern(concatenated);
                            self.pop();
                            self.pop();
                            self.push(Value::STR(interned));
                        } else if let (Value::NUMBER(_), Value::NUMBER(_)) = (x, y) {
                            binary_op!(NUMBER, +, self);
                        } else {
                            return Err(InterpretError::InterpretRuntimeError(
                                "Invalid addition operands".to_string(),
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
                    Opcode::OP_TRUE => self.push(Value::BOOL(true)),
                    Opcode::OP_FALSE => self.push(Value::BOOL(false)),
                    Opcode::OP_NIL => self.push(Value::NIL),
                    Opcode::OP_NOT => {
                        let falsified = Value::BOOL(Value::falsify(&self.pop()));
                        self.push(falsified);
                    }
                    Opcode::OP_EQ => {
                        let a = self.pop();
                        let b = self.pop();
                        self.push(Value::BOOL(Value::values_equal(&a, &b)));
                    }
                    Opcode::OP_GT => {
                        binary_op!(BOOL, >, self);
                    }
                    Opcode::OP_LT => {
                        binary_op!(BOOL, <, self);
                    }
                    Opcode::OP_PRINT => {
                        let val = self.pop();
                        println!("{}", val);
                    }
                    Opcode::OP_POP => {
                        self.pop();
                    }
                    Opcode::OP_DEFINE_GLOBAL(idx) => {
                        let name = Vm::read_constant(&frame, idx).get_string().unwrap();
                        let value = self.pop();
                        self.globals.set(name, value);
                    }
                    Opcode::OP_GET_GLOBAL(idx) => {
                        let name = Vm::read_constant(&frame, idx).get_string().unwrap();
                        if let Some(value) = self.globals.get(name) {
                            self.push(value.clone());
                        } else {
                            return Err(InterpretError::InterpretRuntimeError(
                                "Undefined Variable".to_string(),
                            ));
                        }
                    }
                    Opcode::OP_SET_GLOBAL(idx) => {
                        let name = Vm::read_constant(&frame, idx).get_string().unwrap();
                        let value = self.peek(0);
                        if self.globals.set(name, value.clone()) {
                            self.globals.delete_entry(name);
                            return Err(InterpretError::InterpretRuntimeError(
                                "Undefined Variable".to_string(),
                            ));
                        }
                    }
                    Opcode::OP_GET_LOCAL(slot_index) => {
                        let offset = slot_index + frame.slot;
                        self.push(self.stack[offset].clone());
                    }
                    Opcode::OP_SET_LOCAL(slot_index) => {
                        let offset = slot_index + frame.slot;
                        self.stack[offset] = self.peek(0).clone();
                    }
                    Opcode::OP_JUMP_IF_FALSE(jump_size) => {
                        if Value::is_falsey(self.peek(0)) {
                            frame.ip = frame.ip.offset(jump_size as isize);
                        }
                    }
                    Opcode::OP_JUMP(jump_size) => {
                        frame.ip = frame.ip.offset(jump_size as isize);
                    }
                    Opcode::OP_LOOP(jump_size) => {
                        frame.ip = frame.ip.offset(-(jump_size as isize));
                    }
                    Opcode::OP_CALL(arg_count) => {
                        self.call_value(arg_count)?;
                        frame = &mut *(&mut self.frames[self.frame_count - 1] as *mut CallFrame);
                    }
                }
            }
        }
        // println!("{:?}", self.peek(0));
    }

    fn peek(&self, idx: usize) -> &Value {
        &self.stack[self.stack_top - 1 - idx]
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top].clone()
    }

    fn call_value(&mut self, arg_count: u8) -> Result<(), InterpretError> {
        let callee = self.peek(arg_count.into());
        match callee {
            Value::FUNCTION(x) => {
                return self.call(*x, arg_count);
            }
            _ => return Err(InterpretError::InterpretRuntimeError("Calling uncallable object".to_string())),
        }
    }

    fn call(&mut self, func: GcRef<ObjFunction>, arg_count: u8) -> Result<(), InterpretError> {
        if arg_count != func.arity {
            let msg = format!("Expected {} args but found {}", func.arity, arg_count);
            return Err(InterpretError::InterpretRuntimeError(msg));
        }

        if self.frame_count == Vm::MAX_FRAMES {
            return Err(InterpretError::InterpretRuntimeError("Stack Overflow".to_string()));
        }

        let frame = CallFrame::new(func, self.stack_top - 1 - (arg_count as usize));
        self.frames[self.frame_count] = frame;
        self.frame_count += 1;
        Ok(())
    }
    fn read_constant(frame: &CallFrame, idx: usize) -> Value {
        frame.function.chunk.constants[idx].clone()
    }
}
