use std::{cell::RefCell, rc::Rc};

use crate::{bytecode::Opcode, chunk::Chunk};

use super::Lineno;

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {name} ==");
    for (offset, _) in chunk.code.iter().enumerate() {
        disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) {
    let (opcode, lineno) = chunk.code[offset];
    print!("{:04?} ", offset);
    print!("{:?} ", lineno);
    match opcode {
        Opcode::OP_CONSTANT(idx) => {
            constant_instruction("OPCONSTANT", &chunk, idx, offset);
        }
        _ => {
            println!("{:?}", opcode);
        }
    }
}

fn constant_instruction(name: &str, chunk: &Chunk, idx: usize, offset: usize) {
    let constant = &chunk.constants[idx];
    println!("{name} {idx}");
    println!("{:?}", constant);
}
