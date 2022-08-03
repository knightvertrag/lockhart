use crate::{
    bytecode::Opcode,
    chunk::Chunk,
};

pub fn disassemble_code(chunk: Chunk) {
    for (code, lineno) in chunk.code {
        match code {
            Opcode::OPRETURN => {}
            Opcode::OPCONSTANT(_) => {},
            Opcode::OPNEGATE => {}
        }
    }
}
