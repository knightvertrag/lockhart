use std::{rc::Rc, cell::RefCell};

use crate::{
    bytecode::Opcode,
    chunk::Chunk,
};

pub fn disassemble_code(chunk: &Chunk, name: &str) {
    println!("== {name} ==");
    for (offset, (code, lineno)) in chunk.code.iter().enumerate() {
        match code {
            Opcode::OPRETURN => {
                simple_instruction("OP_RETURN", offset);
            }
            Opcode::OPCONSTANT(idx) => {
                constant_instruction("OP_CONSTANT", &chunk, *idx, offset);
            },
            Opcode::OPNEGATE => {}
            Opcode::OPADD => todo!(),
            Opcode::OPSUBSTRACT => todo!(),
            Opcode::OPMULTIPLY => todo!(),
            Opcode::OPDIVIDE => todo!(),
            Opcode::OPMOD => todo!(),
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

fn constant_instruction(name: &str, chunk: &Chunk, idx: usize, offset: usize) -> usize {
    let constant = &chunk.constants[idx];
    println!("{name} {idx}");
    println!("{:?}", constant);
    offset + 1
}