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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Opcode;

    #[test]
    fn add_constant_returns_index_and_stores_value() {
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::NUMBER(42.0));

        assert_eq!(idx, 0);
        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0].get_number(), Some(42.0));
    }

    #[test]
    fn write_chunk_appends_opcode_and_lineno() {
        let mut chunk = Chunk::new();
        chunk.write_chunk(Opcode::OP_TRUE, Lineno(7));

        assert_eq!(chunk.code.len(), 1);
        match chunk.code[0].0 {
            Opcode::OP_TRUE => {}
            _ => panic!("expected OP_TRUE"),
        }
        assert_eq!(chunk.code[0].1.0, 7);
    }
}
