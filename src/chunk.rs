use crate::{bytecode::Opcode, value::Value};

pub mod disassemble;
#[derive(Debug, Clone, Copy)]
pub struct Lineno(pub usize);
#[derive(Clone)]
pub struct Chunk {
    pub code: Vec<(Opcode, Lineno)>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::<(Opcode, Lineno)>::new(),
            constants: Vec::<Value>::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn write_chunk(&mut self, op: Opcode, lno: Lineno) {
        self.code.push((op, lno));
    }
}
